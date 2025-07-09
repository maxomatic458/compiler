import { createEffect, createSignal, onMount, type Component, For, type JSX } from 'solid-js';

import logo from './logo.svg';
import "./index.css"
import init, * as wasm from '../pkg/compiler';
import { CompileResult } from './types';
import { Terminal } from 'xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { FaSolidChevronDown, FaSolidChevronRight } from 'solid-icons/fa'
import 'xterm/css/xterm.css';

const Resizable: Component<{
  direction: 'horizontal' | 'vertical';
  onResize?: (delta: number) => void;
  style?: JSX.CSSProperties;
}> = (props) => {
  const handleMouseDown = (e: MouseEvent) => {
    const startX = e.clientX;
    const startY = e.clientY;
    
    const handleMouseMove = (e: MouseEvent) => {
      const delta = props.direction === 'horizontal' ? e.clientY - startY : e.clientX - startX;
      props.onResize?.(delta);
    };
    
    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = props.direction === 'horizontal' ? 'row-resize' : 'col-resize';
    document.body.style.userSelect = 'none';
    e.preventDefault();
  };
  
  return (
    <div
      style={{
        'background-color': '#ddd',
        cursor: props.direction === 'horizontal' ? 'row-resize' : 'col-resize',
        'border-radius': '2px',
        ...props.style
      }}
      onMouseDown={handleMouseDown}
    />
  );
};

const defaultSourceCode = `
# A default test program

class Foo<T> {
    data: T,
}

class Bar {
   data: int32,
}

def baz(self) for Bar -> int32 {
    return self.data + 1;
}

def main() -> int32 {
    let foo = Foo { data: 16.4 };
    let bar = Bar { data: 15 };

    if foo.data as int32 == bar.data {
        return 1;
    }

    return 0;
}`

// Tree node component for AST visualization
const TreeNode: Component<{ 
  label: string; 
  value: any; 
  depth: number; 
  isLast?: boolean;
  onSpanHover?: (span: any) => void;
}> = (props) => {
  const [expanded, setExpanded] = createSignal(true);
  
  const isEmpty = (val: any): boolean => {
    if (val == null) return true;
    if (Array.isArray(val)) return val.length === 0;
    if (typeof val === 'object') return Object.keys(val).length === 0;
    return false;
  };

  const isExpandable = (val: any): boolean => {
    return val != null && (Array.isArray(val) || (typeof val === 'object' && Object.keys(val).length > 0));
  };

  const hasSpan = (val: any): boolean => {
    return val && typeof val === 'object' && val.span;
  };

  const renderValue = (val: any) => {
    if (val == null) return <span style={{ color: '#999', 'font-style': 'italic' }}>null</span>;
    if (typeof val === 'string') return <span style={{ color: '#059669' }}>"{val}"</span>;
    if (typeof val === 'number') return <span style={{ color: '#dc2626' }}>{val}</span>;
    if (typeof val === 'boolean') return <span style={{ color: '#7c3aed' }}>{val.toString()}</span>;
    if (Array.isArray(val)) return <span style={{ color: '#666' }}>[{val.length} items]</span>;
    if (typeof val === 'object') return <span style={{ color: '#666' }}>{`{${Object.keys(val).length} fields}`}</span>;
    return String(val);
  };

  const getIcon = (val: any, isExpanded: boolean) => {
    if (isExpandable(val)) {
      return isExpanded ? <FaSolidChevronDown size={16} /> : <FaSolidChevronRight size={16} />;
    }
    if (Array.isArray(val)) {
      return <span style={{ color: '#666', 'font-weight': 'bold' }}>[</span>;
    }
    return null
  };

  const indentWidth = props.depth * 20;
  const isExpand = isExpandable(props.value);
  const hasSpanInfo = hasSpan(props.value);

  return (
    <div>
      <div 
        style={{ 
          display: 'flex', 
          'align-items': 'center', 
          'margin-left': `${indentWidth}px`,
          cursor: isExpand ? 'pointer' : 'default',
          padding: '2px 4px',
          'border-radius': '3px',
          'background-color': hasSpanInfo ? '#fff3cd' : 'transparent'
        }}
        onClick={() => isExpand && setExpanded(!expanded())}
        onMouseEnter={() => hasSpanInfo && props.onSpanHover?.(props.value.span)}
        onMouseLeave={() => hasSpanInfo && props.onSpanHover?.(null)}
      >
        <div style={{ 'margin-right': '8px', display: 'flex', 'align-items': 'center' }}>
          {getIcon(props.value, expanded())}
        </div>
        <span style={{ 'font-weight': '500', color: '#374151' }}>{props.label}:</span>
        <span style={{ 'margin-left': '8px' }}>{renderValue(props.value)}</span>
        {hasSpanInfo && (
          <span style={{ 'margin-left': '8px', 'font-size': '10px', color: '#666', 'font-style': 'italic' }}>
            (has span)
          </span>
        )}
      </div>
      
      {isExpand && expanded() && (
        <div>
          {Array.isArray(props.value) ? (
            <For each={props.value}>
              {(item, index) => (
                <TreeNode 
                  label={`[${index()}]`} 
                  value={item} 
                  depth={props.depth + 1}
                  onSpanHover={props.onSpanHover}
                />
              )}
            </For>
          ) : (
            <For each={Object.entries(props.value)}>
              {([key, value]) => (
                <TreeNode 
                  label={key} 
                  value={value} 
                  depth={props.depth + 1}
                  onSpanHover={props.onSpanHover}
                />
              )}
            </For>
          )}
        </div>
      )}
    </div>
  );
};

// Function to clean up AST data by removing spans (except outer) and empty fields
const cleanASTData = (data: any, keepOuterSpan = false): any => {
  if (data == null) return data;
  
  if (Array.isArray(data)) {
    const cleaned = data.map(item => cleanASTData(item, false)).filter(item => {
      if (item == null) return false;
      if (Array.isArray(item)) return item.length > 0;
      if (typeof item === 'object') return Object.keys(item).length > 0;
      return true;
    });
    return cleaned.length > 0 ? cleaned : undefined;
  }
  
  if (typeof data === 'object') {
    const cleaned: any = {};
    let hasOuterSpan = false;
    
    for (const [key, value] of Object.entries(data)) {
      // Keep outer span if requested
      if (key === 'span' && keepOuterSpan) {
        hasOuterSpan = true;
        cleaned[key] = value;
        continue;
      }
      
      // Skip spans (except outer)
      if (key === 'span' && !keepOuterSpan) {
        continue;
      }
      
      const cleanedValue = cleanASTData(value, false);
      
      // Skip empty values
      if (cleanedValue != null) {
        if (Array.isArray(cleanedValue) && cleanedValue.length === 0) continue;
        if (typeof cleanedValue === 'object' && Object.keys(cleanedValue).length === 0) continue;
        cleaned[key] = cleanedValue;
      }
    }
    
    return Object.keys(cleaned).length > 0 ? cleaned : undefined;
  }
  
  return data;
};

function App() {
  const [sourceCode, setSourceCode] = createSignal<string>(defaultSourceCode);
  const [llvmIr, setLlvmIr] = createSignal<string>("");
  const [wasmInitialized, setWasmInitialized] = createSignal<boolean>(false);
  const [compileResult, setCompileResult] = createSignal<CompileResult | null>(null);
  const [hoveredSpan, setHoveredSpan] = createSignal<any>(null);
  const [terminal, setTerminal] = createSignal<Terminal | null>(null);
  const [errors, setErrors] = createSignal<string[]>([]);
  const [autoCompile, setAutoCompile] = createSignal<boolean>(true);

  // Grid layout state
  const [columns, setColumns] = createSignal('1fr 5px 1fr');
  const [rows, setRows] = createSignal('1fr 5px 1fr 5px 300px');

  let terminalRef: HTMLDivElement | undefined;
  let codeInputRef: HTMLTextAreaElement | undefined;
  let containerRef: HTMLDivElement | undefined;

  const compileCode = () => {
    if (!wasmInitialized()) return; // Don't compile until WASM is initialized
    
    try {
      const result: CompileResult = wasm.compile(sourceCode());

      setCompileResult(result);
      console.log(result.ast);
      console.log(JSON.stringify(result.ast, null, 2));
      setLlvmIr(result.ir || "");
      setErrors([]);
      
      // Clear terminal and show success
      const term = terminal();
      if (term) {
        term.clear();
        term.writeln('\x1b[32m✓ Compilation successful\x1b[0m');
        // term.writeln(`Tokens: ${result.tokens ? result.tokens.length : 0}`);
        // term.writeln(`AST functions: ${result.ast ? Object.keys(result.ast.functions || {}).length : 0}`);
        // term.writeln(`IR length: ${result.ir ? result.ir.length : 0}`);
      }
    } catch (error: any) {
      console.error("Error during compilation:", error);
      setErrors([error.message || error.toString()]);
      
      // Show error in terminal
      const term = terminal();
      if (term) {
        term.clear();
        term.writeln('\x1b[31m✗ Compilation failed:\x1b[0m');
        
        // The error message should already be formatted with ANSI codes
        // Split by lines and write each line separately
        const errorMessage = error.message || error.toString();
        const lines = errorMessage.split('\n');
        lines.forEach((line: string) => {
          term.writeln(line);
        });
      }
    }
  };

  const handleVerticalResize = (delta: number) => {
    if (!containerRef) return;
    
    const rect = containerRef.getBoundingClientRect();
    const relativeX = Math.max(0.1, Math.min(0.9, (rect.width * 0.5 + delta) / rect.width));
    setColumns(`${relativeX}fr 5px ${1 - relativeX}fr`);
  };

  const handleHorizontalResize = (delta: number, isTerminal = false) => {
    if (!containerRef) return;
    
    const rect = containerRef.getBoundingClientRect();
    const currentRows = rows().split(' ');
    
    if (isTerminal) {
      // Resize terminal
      const terminalHeight = Math.max(100, 300 - delta);
      setRows(`${currentRows[0]} 5px ${currentRows[2]} 5px ${terminalHeight}px`);
    } else {
      // Resize between top and middle
      const relativeY = Math.max(0.1, Math.min(0.8, (rect.height * 0.33 + delta) / rect.height));
      setRows(`${relativeY}fr 5px ${0.67 - relativeY}fr 5px 300px`);
    }
  };

  onMount(async () => {
    try {
      await init(); // Initialize the WASM module
      setWasmInitialized(true);
      console.log("WASM module initialized successfully");
      
      // Initialize terminal
      if (terminalRef) {
        const term = new Terminal({
          theme: {
            background: '#1e1e1e',
            foreground: '#ffffff',
            red: '#ff6b6b',
            green: '#51cf66',
            yellow: '#ffd93d',
            blue: '#74c0fc',
            magenta: '#d0bfff',
            cyan: '#8ce99a',
            brightRed: '#ff8787',
            brightGreen: '#69db7c',
            brightYellow: '#ffec99',
            brightBlue: '#91a7ff',
            brightMagenta: '#e599f7',
            brightCyan: '#b2f2bb',
          },
          fontSize: 14,
          fontFamily: 'Monaco, "Cascadia Code", "Segoe UI Mono", Consolas, "Courier New", monospace',
          allowTransparency: true,
          convertEol: true,
        });
        
        const fitAddon = new FitAddon();
        const webLinksAddon = new WebLinksAddon();
        
        term.loadAddon(fitAddon);
        term.loadAddon(webLinksAddon);
        
        term.open(terminalRef);
        fitAddon.fit();
        
        
        setTerminal(term);
        
        // Resize terminal when window resizes
        const resizeObserver = new ResizeObserver(() => {
          fitAddon.fit();
        });
        resizeObserver.observe(terminalRef);
      }
    } catch (error) {
      console.error("Failed to initialize WASM module:", error);
    }
  });

  createEffect(() => {
    if (!autoCompile()) return; // Don't compile if auto-compile is disabled
    compileCode();
  });

  const highlightCodeAtSpan = (span: any) => {
    if (!codeInputRef || !span) return;
    
    const textarea = codeInputRef;
    const start = span.start.abs;
    const end = span.end.abs;
    
    textarea.focus();
    textarea.setSelectionRange(start, end);
  };

  return (
    <div
      ref={containerRef}
      style={{
        height: '100vh',
        display: 'grid',
        'grid-template-columns': columns(),
        'grid-template-rows': rows(),
        gap: '0',
        padding: '10px',
        'background-color': '#f5f5f5',
        overflow: 'hidden'
      }}
    >
      {/* Code Input Area */}
      <div style={{ 
        'background-color': 'white', 
        border: '1px solid #ccc', 
        padding: '10px',
        'border-radius': '4px',
        overflow: 'hidden',
        display: 'flex',
        'flex-direction': 'column'
      }}>
        <div style={{ 
          display: 'flex', 
          'align-items': 'center', 
          'justify-content': 'space-between',
          'margin-bottom': '10px'
        }}>
          <h3 style={{ margin: '0' }}>Source Code</h3>
          <div style={{ display: 'flex', 'align-items': 'center', gap: '12px' }}>
            <label style={{ 
              display: 'flex', 
              'align-items': 'center', 
              gap: '6px',
              'font-size': '14px',
              cursor: 'pointer'
            }}>
              <input
                type="checkbox"
                checked={autoCompile()}
                onChange={(e) => setAutoCompile(e.currentTarget.checked)}
                style={{ margin: '0' }}
              />
              Auto-compile
            </label>
            <button
              onClick={compileCode}
              disabled={!wasmInitialized()}
              style={{
                padding: '6px 12px',
                'background-color': '#007acc',
                color: 'white',
                border: 'none',
                'border-radius': '4px',
                cursor: wasmInitialized() ? 'pointer' : 'not-allowed',
                'font-size': '14px',
                opacity: wasmInitialized() ? '1' : '0.5'
              }}
            >
              Compile
            </button>
          </div>
        </div>
        {!wasmInitialized() ? (
          <p>Loading WASM module...</p>
        ) : (
          <textarea
            ref={codeInputRef}
            value={sourceCode()}
            onInput={(e) => setSourceCode(e.currentTarget.value)}
            style={{ 
              width: '100%', 
              flex: '1',
              'font-family': 'monospace',
              'font-size': '14px',
              border: '1px solid #ddd',
              padding: '8px',
              'border-radius': '4px',
              resize: 'none'
            }}
            placeholder="Enter your code here..."
          />
        )}
      </div>

      {/* Vertical resize handle between code and tokens */}
      <Resizable 
        direction="vertical" 
        onResize={handleVerticalResize}
      />

      {/* Tokens Area */}
      <div style={{ 
        'background-color': 'white', 
        border: '1px solid #ccc', 
        padding: '10px',
        'border-radius': '4px',
        overflow: 'hidden',
        display: 'flex',
        'flex-direction': 'column'
      }}>
        <h3 style={{ margin: '0 0 10px 0' }}>Tokens ({compileResult()?.tokens?.length || 0})</h3>
        <div style={{ flex: '1', overflow: 'auto' }}>
          {compileResult()?.tokens && compileResult()!.tokens.length > 0 ? (
            <For each={compileResult()!.tokens}>
              {(token, index) => (
                <div 
                  style={{ 
                    padding: '4px 8px', 
                    margin: '2px 0',
                    'background-color': hoveredSpan() === token.span ? '#e3f2fd' : '#f9f9f9',
                    border: '1px solid #eee',
                    'border-radius': '3px',
                    cursor: 'pointer',
                    'font-family': 'monospace',
                    'font-size': '12px'
                  }}
                  onMouseEnter={() => { 
                    setHoveredSpan(token.span);
                    highlightCodeAtSpan(token.span)
                  }}
                  onMouseLeave={() => setHoveredSpan(null)}
                >
                  <strong>{JSON.stringify(token.value)}</strong>
                  <br />
                  <small style={{ color: '#666' }}>
                    {token.span?.start?.row}:{token.span?.start?.column} - {token.span?.end?.row}:{token.span?.end?.column}
                  </small>
                </div>
              )}
            </For>
          ) : (
            <p style={{ color: '#666', 'font-style': 'italic' }}>No tokens available</p>
          )}
        </div>
      </div>

      {/* Horizontal resize handle between top and middle rows */}
      <Resizable 
        direction="horizontal" 
        style={{ 'grid-column': '1 / -1' }}
        onResize={(delta) => handleHorizontalResize(delta, false)}
      />

      {/* AST Visualization Area */}
      <div style={{ 
        'background-color': 'white', 
        border: '1px solid #ccc', 
        padding: '10px',
        'border-radius': '4px',
        overflow: 'hidden',
        display: 'flex',
        'flex-direction': 'column'
      }}>
        <h3 style={{ margin: '0 0 10px 0' }}>Abstract Syntax Tree</h3>
        <div style={{ 
          flex: '1',
          overflow: 'auto', 
          'font-family': 'monospace',
          'font-size': '12px'
        }}>
          {compileResult()?.ast ? (
            <div>
              <div style={{ 'margin-bottom': '10px' }}>
                <strong>Data Types:</strong>
                {compileResult()!.ast.custom_types && Object.keys(compileResult()!.ast.custom_types).length > 0 ? (
                  <div style={{ 'margin-left': '10px' }}>
                    <For each={Object.entries(compileResult()!.ast.custom_types)}>
                      {([key, value]) => (
                        <TreeNode 
                          label={key} 
                          value={cleanASTData(value, true)} 
                          depth={0}
                          onSpanHover={(span) => {
                            setHoveredSpan(span);
                            highlightCodeAtSpan(span);
                          }}
                        />
                      )}
                    </For>
                  </div>
                ) : (
                  <span style={{ color: '#666' }}> None</span>
                )}
              </div>
              
              <div style={{ 'margin-bottom': '10px' }}>
                <strong>Functions:</strong>
                {compileResult()!.ast.functions && Object.keys(compileResult()!.ast.functions).length > 0 ? (
                  <div style={{ 'margin-left': '10px' }}>
                    <For each={Object.entries(compileResult()!.ast.functions)}>
                      {([key, value]) => (
                        <TreeNode 
                          label={key} 
                          value={cleanASTData(value, true)} 
                          depth={0}
                          onSpanHover={(span) => {
                            setHoveredSpan(span);
                            highlightCodeAtSpan(span);
                          }}
                        />
                      )}
                    </For>
                  </div>
                ) : (
                  <span style={{ color: '#666' }}> None</span>
                )}
              </div>
              
              <div>
                <strong>Require Main:</strong> {compileResult()!.ast.require_main ? 'Yes' : 'No'}
              </div>
            </div>
          ) : (
            <p style={{ color: '#666', 'font-style': 'italic' }}>No AST available</p>
          )}
        </div>
      </div>

      {/* Vertical resize handle between AST and LLVM IR */}
      <Resizable 
        direction="vertical" 
        onResize={handleVerticalResize}
      />

      {/* LLVM IR Area */}
      <div style={{ 
        'background-color': 'white', 
        border: '1px solid #ccc', 
        padding: '10px',
        'border-radius': '4px',
        overflow: 'hidden',
        display: 'flex',
        'flex-direction': 'column'
      }}>
        <h3 style={{ margin: '0 0 10px 0' }}>LLVM IR ({llvmIr().length} characters)</h3>
        <pre style={{ 
          'background-color': '#f8f8f8', 
          padding: '10px', 
          overflow: 'auto',
          flex: '1',
          margin: '0',
          'font-family': 'monospace',
          'font-size': '12px',
          'white-space': 'pre-wrap'
        }}>
          {llvmIr() || <span style={{ color: '#666', 'font-style': 'italic' }}>No LLVM IR generated</span>}
        </pre>
      </div>

      {/* Horizontal resize handle between middle and terminal */}
      <Resizable 
        direction="horizontal" 
        style={{ 'grid-column': '1 / -1' }}
        onResize={(delta) => handleHorizontalResize(delta, true)}
      />

      {/* Terminal */}
      <div style={{ 
        'background-color': '#1e1e1e',
        border: '1px solid #333',
        padding: '10px',
        'border-radius': '4px',
        'grid-column': '1 / -1',
        display: 'flex',
        'flex-direction': 'column',
        overflow: 'hidden'
      }}>
        <h3 style={{ color: 'white', margin: '0 0 10px 0' }}>Compiler output</h3>
        <div 
          ref={terminalRef}
          style={{ 
            flex: '1',
            width: '100%'
          }}
        />
      </div>
    </div>
  );
};

export default App;
