use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    core::{should_activate, ActiveNeuron, Synapse, SynapseType, Task, TaskType, VirtualNeuron},
    energy::EnergyBudget,
    routing::{activation_budget::enforce_sparse_budget, route_efficiency::route_efficiency_bonus},
    storage::DiskStore,
};

#[derive(Debug, Clone)]
pub struct RoutingResult {
    pub active_neurons: Vec<ActiveNeuron>,
    pub loaded_synapses: Vec<Synapse>,
    pub active_neuron_records: Vec<VirtualNeuron>,
}

pub struct Router;

impl Router {
    pub fn route(store: &DiskStore, task: &Task, budget: &EnergyBudget) -> Result<RoutingResult> {
        let seed_ids = seed_ids(store, task)?;
        let mut scores: BTreeMap<String, (f32, Vec<String>, usize, f32)> = BTreeMap::new();
        let mut loaded_synapses = Vec::new();
        let mut seen_synapses = BTreeSet::new();

        for id in &seed_ids {
            if let Ok(neuron) = store.load_neuron(id) {
                let relevance = task_relevance(task, &neuron);
                scores.insert(
                    id.clone(),
                    (
                        neuron.base_activation + relevance,
                        vec![format!("seeded by {:?}", task.task_type)],
                        0,
                        0.1,
                    ),
                );
            }
        }

        let mut frontier = seed_ids;
        for _ in 0..3 {
            let mut next_frontier = BTreeSet::new();
            for id in frontier {
                let outgoing = store
                    .read_outgoing_synapse_ids(&id)?
                    .into_iter()
                    .take(budget.max_active_synapses_per_neuron);
                for synapse_id in outgoing {
                    if !seen_synapses.insert(synapse_id.clone()) {
                        continue;
                    }
                    let synapse = store.load_synapse(&synapse_id)?;
                    let target = store.load_neuron(&synapse.to)?;
                    let signed = match synapse.synapse_type {
                        SynapseType::Inhibitory => -(synapse.weight.abs() * synapse.confidence),
                        _ => synapse.weight * synapse.confidence,
                    };
                    let entry = scores.entry(target.id.clone()).or_insert_with(|| {
                        (
                            target.base_activation + task_relevance(task, &target),
                            Vec::new(),
                            0,
                            0.0,
                        )
                    });
                    entry.0 += signed + synapse.success_score * 0.05 - synapse.energy_cost * 0.2
                        + route_efficiency_bonus(store, &synapse.id);
                    entry
                        .1
                        .push(format!("via {} from {}", synapse.id, synapse.from));
                    entry.2 += 1;
                    entry.3 += synapse.energy_cost;
                    next_frontier.insert(target.id.clone());
                    loaded_synapses.push(synapse);
                }
            }
            if next_frontier.is_empty() {
                break;
            }
            frontier = next_frontier.into_iter().collect();
        }

        let mut active = Vec::new();
        let mut records = Vec::new();
        for (id, (score, reasons, loaded_count, energy)) in scores {
            let neuron = store.load_neuron(&id)?;
            if should_activate(score, neuron.threshold) {
                active.push(ActiveNeuron {
                    id: id.clone(),
                    activation: score,
                    threshold: neuron.threshold,
                    reasons,
                    loaded_synapse_count: loaded_count,
                    estimated_energy_cost: energy.max(0.1),
                });
                records.push(neuron);
            }
        }
        let active = enforce_sparse_budget(active, budget);
        let active_ids: BTreeSet<_> = active.iter().map(|neuron| neuron.id.clone()).collect();
        records.retain(|record| active_ids.contains(&record.id));

        Ok(RoutingResult {
            active_neurons: active,
            loaded_synapses,
            active_neuron_records: records,
        })
    }
}

fn seed_ids(store: &DiskStore, task: &Task) -> Result<Vec<String>> {
    let mut ids = BTreeSet::new();
    let index = store.task_type_index()?;
    let key = task_type_key(&task.task_type);
    if let Some(indexed) = index.0.get(key) {
        ids.extend(indexed.iter().cloned());
    }
    if matches!(task.task_type, TaskType::Code) && task.input.to_lowercase().contains("create") {
        if let Some(indexed) = index.0.get("FileOperation") {
            ids.extend(indexed.iter().cloned());
        }
    }
    let label_index = store.label_index()?;
    for word in task.input.split(|c: char| !c.is_ascii_alphanumeric()) {
        let normalized = word.to_lowercase();
        if let Some(id) = label_index.0.get(&normalized) {
            ids.insert(id.clone());
        }
    }
    if ids.is_empty() {
        ids.insert("task_chat".to_string());
    }
    Ok(ids.into_iter().collect())
}

fn task_type_key(task_type: &TaskType) -> &'static str {
    match task_type {
        TaskType::Chat => "Chat",
        TaskType::Code => "Code",
        TaskType::FileOperation => "FileOperation",
        TaskType::Planning => "Planning",
        TaskType::Reasoning => "Reasoning",
        TaskType::MemoryQuery => "MemoryQuery",
        TaskType::ToolUse => "ToolUse",
        TaskType::Unknown => "Unknown",
    }
}

fn task_relevance(task: &Task, neuron: &VirtualNeuron) -> f32 {
    let label = neuron.label.to_lowercase();
    let input = task.input.to_lowercase();
    let mut score = 0.0;
    for token in label.split(|c: char| !c.is_ascii_alphanumeric()) {
        if !token.is_empty() && input.contains(token) {
            score += 0.25;
        }
    }
    if label.contains(&task_type_key(&task.task_type).to_lowercase()) {
        score += 0.8;
    }
    score
}
