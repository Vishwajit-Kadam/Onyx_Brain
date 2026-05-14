use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    core::{MemoryId, NeuronId, SynapseId, VirtualNeuron},
    experts::ExpertStatsIndex,
    memory::MemoryItem,
    storage::{load_json, save_json, try_load_json},
};

#[derive(Debug, Clone)]
pub struct DataPaths {
    pub root: PathBuf,
    pub data: PathBuf,
    pub neurons: PathBuf,
    pub synapses: PathBuf,
    pub memories: PathBuf,
    pub projects: PathBuf,
    pub logs: PathBuf,
    pub indexes: PathBuf,
    pub outgoing_synapses: PathBuf,
    pub cache: PathBuf,
    pub plan_cache: PathBuf,
    pub habits: PathBuf,
    pub journal: PathBuf,
    pub snapshots: PathBuf,
    pub transactions: PathBuf,
    pub sessions: PathBuf,
    pub recovery: PathBuf,
    pub conversations: PathBuf,
    pub conversation_memory: PathBuf,
    pub config: PathBuf,
    pub executive: PathBuf,
    pub events: PathBuf,
    pub sandbox: PathBuf,
}

impl DataPaths {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let data = root.join("data");
        let indexes = data.join("indexes");
        Self {
            neurons: data.join("neurons"),
            synapses: data.join("synapses"),
            memories: data.join("memories"),
            projects: data.join("projects"),
            logs: data.join("logs"),
            outgoing_synapses: indexes.join("outgoing_synapses"),
            cache: data.join("cache"),
            plan_cache: data.join("cache").join("plans"),
            habits: data.join("habits"),
            journal: data.join("journal"),
            snapshots: data.join("snapshots"),
            transactions: data.join("transactions"),
            sessions: data.join("sessions"),
            recovery: data.join("recovery"),
            conversations: data.join("conversations"),
            conversation_memory: data.join("conversation_memory"),
            config: data.join("config"),
            executive: data.join("executive"),
            events: data.join("events"),
            sandbox: root.join("sandbox"),
            indexes,
            data,
            root,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        for dir in [
            &self.neurons,
            &self.synapses,
            &self.memories,
            &self.projects,
            &self.logs,
            &self.indexes,
            &self.outgoing_synapses,
            &self.cache,
            &self.plan_cache,
            &self.habits,
            &self.journal,
            &self.snapshots,
            &self.transactions,
            &self.sessions,
            &self.recovery,
            &self.conversations,
            &self.conversation_memory,
            &self.config,
            &self.executive,
            &self.events,
            &self.sandbox,
        ] {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LabelIndex(pub BTreeMap<String, NeuronId>);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskTypeIndex(pub BTreeMap<String, Vec<NeuronId>>);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryTagIndex(pub BTreeMap<String, Vec<MemoryId>>);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryKeywordIndex(pub BTreeMap<String, Vec<MemoryId>>);

#[derive(Debug, Clone)]
pub struct DiskStore {
    pub paths: DataPaths,
}

impl DiskStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            paths: DataPaths::new(root),
        }
    }

    pub fn ensure_layout(&self) -> Result<()> {
        self.paths.ensure()
    }

    pub fn neuron_path(&self, id: &str) -> PathBuf {
        self.paths.neurons.join(format!("{id}.json"))
    }

    pub fn synapse_path(&self, id: &str) -> PathBuf {
        self.paths.synapses.join(format!("{id}.json"))
    }

    pub fn memory_path(&self, id: &str) -> PathBuf {
        self.paths.memories.join(format!("{id}.json"))
    }

    pub fn save_neuron(&self, neuron: &VirtualNeuron) -> Result<()> {
        save_json(&self.neuron_path(&neuron.id), neuron)
    }

    pub fn load_neuron(&self, id: &str) -> Result<VirtualNeuron> {
        load_json(&self.neuron_path(id))
    }

    pub fn save_synapse(&self, synapse: &crate::core::Synapse) -> Result<()> {
        save_json(&self.synapse_path(&synapse.id), synapse)
    }

    pub fn load_synapse(&self, id: &str) -> Result<crate::core::Synapse> {
        load_json(&self.synapse_path(id))
    }

    pub fn save_memory(&self, memory: &MemoryItem) -> Result<()> {
        save_json(&self.memory_path(&memory.id), memory)?;
        self.index_memory(memory)
    }

    pub fn load_memory(&self, id: &str) -> Result<MemoryItem> {
        load_json(&self.memory_path(id))
    }

    pub fn memory_files(&self) -> Result<Vec<PathBuf>> {
        list_json_files(&self.paths.memories)
    }

    pub fn synapse_files(&self) -> Result<Vec<PathBuf>> {
        list_json_files(&self.paths.synapses)
    }

    pub fn read_outgoing_synapse_ids(&self, neuron_id: &str) -> Result<Vec<SynapseId>> {
        let path = self
            .paths
            .outgoing_synapses
            .join(format!("{neuron_id}.json"));
        Ok(try_load_json(&path)?.unwrap_or_default())
    }

    pub fn write_outgoing_synapse_ids(&self, neuron_id: &str, ids: &[SynapseId]) -> Result<()> {
        let path = self
            .paths
            .outgoing_synapses
            .join(format!("{neuron_id}.json"));
        save_json(&path, &ids)
    }

    pub fn label_index(&self) -> Result<LabelIndex> {
        let path = self.paths.indexes.join("label_index.json");
        Ok(try_load_json(&path)?.unwrap_or_default())
    }

    pub fn save_label_index(&self, index: &LabelIndex) -> Result<()> {
        save_json(&self.paths.indexes.join("label_index.json"), index)
    }

    pub fn task_type_index(&self) -> Result<TaskTypeIndex> {
        let path = self.paths.indexes.join("task_type_index.json");
        Ok(try_load_json(&path)?.unwrap_or_default())
    }

    pub fn save_task_type_index(&self, index: &TaskTypeIndex) -> Result<()> {
        save_json(&self.paths.indexes.join("task_type_index.json"), index)
    }

    pub fn memory_tag_index(&self) -> Result<MemoryTagIndex> {
        let path = self.paths.indexes.join("memory_tags.json");
        Ok(try_load_json(&path)?.unwrap_or_default())
    }

    pub fn save_memory_tag_index(&self, index: &MemoryTagIndex) -> Result<()> {
        save_json(&self.paths.indexes.join("memory_tags.json"), index)
    }

    pub fn memory_keyword_index(&self) -> Result<MemoryKeywordIndex> {
        let path = self.paths.indexes.join("memory_keywords.json");
        Ok(try_load_json(&path)?.unwrap_or_default())
    }

    pub fn save_memory_keyword_index(&self, index: &MemoryKeywordIndex) -> Result<()> {
        save_json(&self.paths.indexes.join("memory_keywords.json"), index)
    }

    pub fn expert_stats_index(&self) -> Result<ExpertStatsIndex> {
        let path = self.paths.indexes.join("expert_stats.json");
        Ok(try_load_json(&path)?.unwrap_or_default())
    }

    pub fn save_expert_stats_index(&self, index: &ExpertStatsIndex) -> Result<()> {
        save_json(&self.paths.indexes.join("expert_stats.json"), index)
    }

    pub fn save_log<T: Serialize>(&self, name: &str, value: &T) -> Result<()> {
        save_json(&self.paths.logs.join(format!("{name}.json")), value)
    }

    pub fn list_log_files(&self) -> Result<Vec<PathBuf>> {
        list_json_files(&self.paths.logs)
    }

    pub fn write_checkpoint<T: Serialize>(&self, task_id: &str, value: &T) -> Result<()> {
        save_json(
            &self
                .paths
                .projects
                .join(format!("{task_id}_checkpoint.json")),
            value,
        )
    }

    fn index_memory(&self, memory: &MemoryItem) -> Result<()> {
        let mut tag_index = self.memory_tag_index()?;
        for tag in &memory.tags {
            push_unique(
                tag_index.0.entry(normalize_token(tag)).or_default(),
                memory.id.clone(),
            );
        }
        self.save_memory_tag_index(&tag_index)?;

        let mut keyword_index = self.memory_keyword_index()?;
        for keyword in memory_keywords(memory) {
            push_unique(
                keyword_index.0.entry(keyword).or_default(),
                memory.id.clone(),
            );
        }
        self.save_memory_keyword_index(&keyword_index)
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
        values.sort();
    }
}

fn normalize_token(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

fn memory_keywords(memory: &MemoryItem) -> Vec<String> {
    let text = format!("{} {} {}", memory.title, memory.summary, memory.content);
    let mut words = text
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|word| word.len() > 2)
        .map(normalize_token)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    words.sort();
    words.dedup();
    words
}

pub fn list_json_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("listing {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

pub fn load_memory_from_path(path: &Path) -> Result<MemoryItem> {
    load_json(path)
}

pub fn load_synapse_from_path(path: &Path) -> Result<crate::core::Synapse> {
    load_json(path)
}

pub fn memory_id_from_file(path: &Path) -> Option<MemoryId> {
    path.file_stem()?.to_str().map(ToOwned::to_owned)
}
