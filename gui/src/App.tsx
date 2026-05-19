import { useState, useRef, useEffect } from 'react';
import { 
  Plus, 
  Search, 
  Clock, 
  FolderOpen, 
  Filter, 
  Settings, 
  ChevronDown,
  ArrowUp,
  Mic,
  Library,
  Box,
  Monitor,
  PenSquare,
  Globe,
  MonitorPlay,
  Palette,
  Bell,
  RefreshCw,
  X,
  Sidebar,
  MessagesSquare,
  Sparkles
} from 'lucide-react';
import './index.css';

interface Message {
  id: number;
  text: string;
  sender: 'user' | 'bot';
}

function App() {
  const [inputText, setInputText] = useState('');
  const [messages, setMessages] = useState<Message[]>([]);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSend = () => {
    if (!inputText.trim()) return;
    
    // Add user message
    const userMsg: Message = { id: Date.now(), text: inputText, sender: 'user' };
    setMessages(prev => [...prev, userMsg]);
    setInputText('');
    
    // Simulate bot response
    setTimeout(() => {
      const botMsg: Message = { 
        id: Date.now() + 1, 
        text: `Onyx Brain acknowledges: "${userMsg.text}". I am ready to process this task via deterministic pathways.`, 
        sender: 'bot' 
      };
      setMessages(prev => [...prev, botMsg]);
    }, 1000);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const suggestionCards = [
    "Build your personal website or portfolio without writing a single line of code.",
    "Set up automated monitoring for any topic, competitor, or keyword.",
    "Track your health and fitness with a dashboard and a personalized plan."
  ];

  return (
    <div className="app-container">
      {/* Sidebar */}
      {isSidebarOpen && (
        <aside className="sidebar">
          <div className="sidebar-header">
            <div className="logo">
              <Sparkles size={20} color="var(--accent-blue)" />
              Onyx
            </div>
            <button className="icon-btn" onClick={() => setIsSidebarOpen(false)}>
              <Sidebar size={18} />
            </button>
          </div>

          <button className="btn-new-task" onClick={() => setMessages([])}>
            <PenSquare size={16} />
            New task
          </button>

          <nav className="sidebar-nav">

            <a href="#" className="nav-item">
              <Clock size={16} />
              Scheduled
            </a>
            <a href="#" className="nav-item">
              <Search size={16} />
              Search
            </a>
            <a href="#" className="nav-item">
              <Library size={16} />
              Library
            </a>
          </nav>

          <div className="sidebar-section">
            <div className="section-title">
              <span>Projects</span>
              <button><Plus size={14} /></button>
            </div>
            <a href="#" className="nav-item">
              <FolderOpen size={16} />
              New project
            </a>
          </div>

          <div className="sidebar-section" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <div className="section-title">
              <span>All tasks</span>
              <button><Filter size={14} /></button>
            </div>
            {messages.length === 0 ? (
              <div className="empty-state">
                <MessagesSquare size={48} style={{ opacity: 0.2, marginBottom: '12px' }} />
                <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Create a new task to get started</div>
              </div>
            ) : (
              <a href="#" className="nav-item active">
                <PenSquare size={16} />
                Current Task
              </a>
            )}
          </div>

          <div className="sidebar-footer">
            <div className="footer-icons">
              <button><Settings size={16} /></button>
              <button><Box size={16} /></button>
              <button><Monitor size={16} /></button>
            </div>
            <span style={{ fontSize: '0.75rem' }}>from <strong>Meta</strong></span>
          </div>
        </aside>
      )}

      {/* Main Content */}
      <main className="main-content">
        <header className="topbar">
          {!isSidebarOpen && (
            <button className="icon-btn" onClick={() => setIsSidebarOpen(true)} style={{ marginRight: '16px' }}>
              <Sidebar size={18} />
            </button>
          )}
          
          <div className="model-selector clickable" onClick={() => setIsDropdownOpen(!isDropdownOpen)}>
            Onyx_Brain <ChevronDown size={16} />
            {isDropdownOpen && (
              <div className="dropdown-menu">
                <button className="dropdown-item">Onyx_Brain v0.0.4</button>
                <button className="dropdown-item">Settings</button>
              </div>
            )}
          </div>
          
          <div className="topbar-right">
            <button className="icon-btn"><Bell size={18} /></button>
            <div className="avatar">U</div>
          </div>
        </header>

        <div className="center-area">
          {messages.length === 0 ? (
            <>
              <div className="plan-pill">
                <button className="plan-option active">Free plan</button>
                <button className="plan-option upgrade">Upgrade</button>
              </div>

              <h1 className="greeting">What can I do for you?</h1>
            </>
          ) : (
            <div className="chat-messages">
              {messages.map((msg) => (
                <div key={msg.id} className={`message ${msg.sender}`}>
                  {msg.sender === 'bot' && (
                    <div className="message-avatar">
                      <Box size={16} />
                    </div>
                  )}
                  <div className="message-content">
                    {msg.text}
                  </div>
                  {msg.sender === 'user' && (
                    <div className="message-avatar">U</div>
                  )}
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>
          )}

          <div className="input-container" style={{ marginTop: messages.length > 0 ? 'auto' : '0' }}>
            <textarea 
              placeholder="Assign a task or ask anything" 
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              onKeyDown={handleKeyDown}
            />
            
            <div className="input-actions">
              <div className="input-actions-left">
                <button className="icon-btn"><Plus size={18} /></button>
                <button className="icon-btn"><Sparkles size={18} /></button>
              </div>
              <div className="input-actions-right">
                <button className="icon-btn"><Plus size={18} /></button>
                <button className="icon-btn"><Mic size={18} /></button>
                <button 
                  className={`send-btn clickable ${inputText.trim() ? 'active' : ''}`}
                  onClick={handleSend}
                  disabled={!inputText.trim()}
                >
                  <ArrowUp size={16} />
                </button>
              </div>
            </div>
          </div>

          {messages.length === 0 && (
            <div className="suggestions-container">
              <div className="suggestions-header">
                <span className="suggestions-title">Suggested for you</span>
                <div className="suggestions-actions">
                  <button className="icon-btn"><RefreshCw size={14} /></button>
                  <button className="icon-btn"><X size={14} /></button>
                </div>
              </div>

              <div className="cards-grid">
                {suggestionCards.map((text, i) => (
                  <button key={i} className="card clickable" onClick={() => setInputText(text)}>
                    {text}
                  </button>
                ))}
              </div>
            </div>
          )}

          {messages.length === 0 && (
            <>
              <div className="pills-row">
                <button className="pill-btn clickable" onClick={() => setInputText("Create slides")}>
                  <MonitorPlay size={16} /> Create slides
                </button>
                <button className="pill-btn clickable" onClick={() => setInputText("Build website")}>
                  <Globe size={16} /> Build website
                </button>
                <button className="pill-btn clickable" onClick={() => setInputText("Develop desktop apps")}>
                  <Monitor size={16} /> Develop desktop apps
                </button>
                <button className="pill-btn clickable" onClick={() => setInputText("Design")}>
                  <Palette size={16} /> Design
                </button>
                <button className="pill-btn clickable">
                  More
                </button>
              </div>

              <div className="banner clickable">
                <div className="banner-content">
                  <h3>Create Skills</h3>
                  <p>Automate your workflow with custom Skills for repetitive tasks.</p>
                </div>
                <div className="banner-graphic">
                  <div className="graphic-line">
                    <div className="graphic-icon" style={{ backgroundColor: '#10b981' }}></div>
                  </div>
                  <div className="graphic-line">
                    <div className="graphic-icon" style={{ backgroundColor: '#3b82f6' }}></div>
                  </div>
                  <div className="graphic-line">
                    <div className="graphic-icon" style={{ backgroundColor: '#f59e0b' }}></div>
                  </div>
                </div>
              </div>
            </>
          )}
        </div>
      </main>
    </div>
  );
}

export default App;
