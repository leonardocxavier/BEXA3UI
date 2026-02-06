# RFC-001: TextInput, Keyboard Dispatch e Terminal Widget

**Status**: Draft
**Autor**: BexaUI Team
**Data**: 2026-02-06

---

## 1. Motivacao

O BexaUI precisa de capacidade de entrada de texto para evoluir de um framework de
visualizacao para um framework interativo completo. O objetivo final e um terminal
embutido (estilo WinSCP), mas isso depende de building blocks que ainda nao existem.

Este RFC define a ordem correta de implementacao, as dependencias entre milestones,
e a arquitetura de cada componente.

---

## 2. Estado Atual

### O que temos
| Recurso             | Status      |
|---------------------|-------------|
| Layout (taffy)      | Completo    |
| Rendering (wgpu)    | Completo    |
| SDF rounded rects   | Completo    |
| Text (glyphon)      | Completo    |
| Scroll (por widget) | Completo    |
| Clipping (scissor)  | Completo    |
| Buttons + hover     | Completo    |
| Signals (reactivo)  | Completo    |
| Focus (Tab/click)   | Completo    |
| Icon Fonts (Nerd)   | Completo    |
| Theme system        | Completo    |

### O que falta para input de texto
| Recurso                       | Status      |
|-------------------------------|-------------|
| Keyboard dispatch por widget  | Nao existe  |
| Cursor de texto (caret)       | Nao existe  |
| Selecao de texto              | Nao existe  |
| TextInput widget              | Nao existe  |
| Clipboard (Ctrl+C/V)          | Nao existe  |
| IME support                   | Nao existe  |

### O que falta para terminal
| Recurso                       | Status      |
|-------------------------------|-------------|
| PTY (pseudo-terminal)         | Nao existe  |
| VT100/ANSI parser             | Nao existe  |
| Grid de caracteres            | Nao existe  |
| Terminal widget               | Nao existe  |

---

## 3. Milestones

```
M8: Keyboard Dispatch     (pre-requisito para tudo)
 |
M9: TextInput Widget      (depende de M8)
 |
M10: Clipboard            (depende de M9)
 |
M11: Terminal Widget       (depende de M8 + M9 + M10)
```

---

## 4. M8 - Keyboard Dispatch por Widget

### Problema
Atualmente o event loop trata teclado de forma centralizada:
- Tab -> foca proximo widget
- Enter/Space -> ativa widget focado
- Escape -> limpa estado ativo

Nenhum evento de teclado chega nos widgets via `handle_event()`. Para um
TextInput funcionar, o widget focado precisa receber `KeyboardInput` e
`Ime` events.

### Solucao

#### 4.1 Novo metodo no Widget trait
```rust
pub trait Widget {
    // ... metodos existentes ...

    /// Chamado quando o widget tem foco e uma tecla e pressionada.
    /// Retorna true se o evento foi consumido.
    fn handle_key_event(&mut self, _event: &KeyEvent) -> bool {
        false
    }
}
```

Usar `KeyEvent` do winit (contem `logical_key`, `text`, `state`, `repeat`).

#### 4.2 Mudancas no event loop (bexa-ui-render/src/lib.rs)
```
WindowEvent::KeyboardInput { event, .. } => {
    if event.state == ElementState::Pressed {
        // 1. Tenta despachar para widget focado
        let consumed = self.dispatch_key_to_focused(&event);

        // 2. Se nao consumido, trata navegacao global
        if !consumed {
            match event.logical_key {
                Key::Named(NamedKey::Tab) => self.focus_next(self.modifiers.shift_key()),
                Key::Named(NamedKey::Escape) => clear_active_widgets(&mut self.root),
                _ => {}
            }
        }
    }
}
```

#### 4.3 Nova funcao dispatch_key_to_focused
```rust
fn dispatch_key_to_focused(&mut self, event: &KeyEvent) -> bool {
    if let Some(idx) = self.focused_index {
        let path = &self.focus_paths[idx];
        if let Some(widget) = widget_mut_at_path(&mut self.root, path) {
            return widget.handle_key_event(event);
        }
    }
    false
}
```

#### 4.4 Mudancas necessarias
| Arquivo                          | Mudanca                                    |
|----------------------------------|--------------------------------------------|
| `bexa-ui-core/src/framework.rs`  | Adicionar `handle_key_event` ao trait       |
| `bexa-ui-render/src/lib.rs`      | Refatorar event loop keyboard              |
| `bexa-ui-render/src/lib.rs`      | Adicionar `dispatch_key_to_focused`        |

#### 4.5 Retrocompatibilidade
- Button continua funcionando (Enter/Space e tratado pelo dispatch global apenas se `handle_key_event` retornar false)
- Tab navigation permanece como fallback global
- Widgets existentes nao precisam implementar `handle_key_event`

### Estimativa de complexidade: Baixa (< 50 linhas)

---

## 5. M9 - TextInput Widget

### Objetivo
Campo de texto editavel com cursor, similar a `<input type="text">` no HTML.

### Arquitetura

```
TextInput
  |-- text: String           (conteudo atual)
  |-- cursor_pos: usize      (posicao do cursor no texto)
  |-- selection: Option<(usize, usize)>  (inicio, fim da selecao)
  |-- signal: SetSignal<String>          (notifica mudancas)
  |-- focused: bool
  |-- metrics: Metrics
  |-- placeholder: Option<String>
```

### 5.1 Renderizacao

```
+--[ Container bg ]-------------------------------------------+
|  padding                                                     |
|  +--[ texto antes do cursor ][ | ][ texto depois ]----------+|
|  padding                                                     |
+--------------------------------------------------------------+
```

- Fundo: retangulo com border radius (Container-like)
- Texto: renderizado via `draw_text()` existente
- Cursor (caret): retangulo fino (1-2px) piscando via timer
- Selecao: retangulo semi-transparente atras do texto selecionado

### 5.2 Cursor piscante

O cursor precisa piscar (~530ms intervalo). Opcoes:

**Opcao A - Frame counter (recomendada)**:
```rust
fn draw(&self, ctx: &mut DrawContext) {
    // cursor_visible alterna a cada N frames
    // Simples, sem dependencia de timer externo
    if self.focused && self.blink_visible() {
        // desenha caret
    }
}
```

Para isso funcionar, o render loop precisa fazer `request_redraw` periodicamente
quando um TextInput esta focado. Adicionar ao event loop:
```rust
Event::AboutToWait => {
    if self.has_focused_input() {
        self.window.request_redraw(); // forca redraw para blink
    }
}
```

**Opcao B - Instant::now()**: Comparar com timestamp do ultimo keystroke.
Mais preciso, mas requer `std::time::Instant` no widget.

Recomendacao: **Opcao B** (Instant-based), pois e independente de frame rate.

### 5.3 Medida de texto (hit testing)

Para posicionar o cursor ao clicar, precisamos saber a largura de cada
caractere. O glyphon ja faz isso internamente (shaping). Podemos:

1. Usar `Buffer::line_layout()` do cosmic-text para obter posicoes de glyph
2. Ou fazer medida simplificada assumindo largura fixa (monospace)

Para a v1 (M9), usaremos **medida simplificada** baseada em `metrics.font_size * 0.6`
como largura media de caractere. Refinamos depois com medida real.

### 5.4 handle_key_event

```rust
fn handle_key_event(&mut self, event: &KeyEvent) -> bool {
    match &event.logical_key {
        // Caracteres digitados
        Key::Character(c) => {
            self.insert_text(c);
            true
        }
        // Teclas especiais
        Key::Named(NamedKey::Backspace) => { self.delete_back(); true }
        Key::Named(NamedKey::Delete) => { self.delete_forward(); true }
        Key::Named(NamedKey::ArrowLeft) => { self.move_cursor(-1); true }
        Key::Named(NamedKey::ArrowRight) => { self.move_cursor(1); true }
        Key::Named(NamedKey::Home) => { self.cursor_pos = 0; true }
        Key::Named(NamedKey::End) => { self.cursor_pos = self.text.len(); true }
        // Enter para formularios (single-line nao consome Enter)
        Key::Named(NamedKey::Enter) => false,
        // Tab nao consumido (permite navegacao)
        Key::Named(NamedKey::Tab) => false,
        _ => false,
    }
}
```

### 5.5 API publica

```rust
let (value, set_value) = create_signal(String::new());

let input = TextInput::new(value.clone(), set_value)
    .with_placeholder("Digite aqui...")
    .with_metrics(Metrics::new(16.0, 22.0))
    .with_text_color([230, 230, 230])
    .with_background([0.15, 0.20, 0.28])
    .with_border_radius(6.0)
    .with_padding(10.0);

WidgetNode::new(input, vec![])
```

### 5.6 Mudancas necessarias

| Arquivo                               | Mudanca                              |
|---------------------------------------|--------------------------------------|
| `bexa-ui-core/src/widgets/text_input.rs` | Novo widget                       |
| `bexa-ui-core/src/widgets/mod.rs`     | Registrar TextInput                  |
| `bexa-ui-core/src/lib.rs`             | Re-exportar TextInput                |
| `bexa-ui/src/lib.rs`                  | Adicionar ao prelude                 |
| `bexa-ui-render/src/lib.rs`           | `AboutToWait` para blink / redraw   |
| `examples/dashboard.rs`               | Secao de demo TextInput              |

### Estimativa de complexidade: Media (~200-300 linhas)

---

## 6. M10 - Clipboard (Ctrl+C / Ctrl+V)

### Problema
O TextInput precisa de copiar/colar para ser usavel.

### Solucao
Usar a crate `arboard` (cross-platform clipboard).

```toml
# bexa-ui-core/Cargo.toml
[dependencies]
arboard = "3"
```

### 6.1 Integracao no TextInput

```rust
fn handle_key_event(&mut self, event: &KeyEvent) -> bool {
    // Detecta Ctrl/Cmd
    let ctrl = /* modifiers.control_key() ou command no mac */;

    if ctrl {
        match &event.logical_key {
            Key::Character(c) if c == "c" => { self.copy_selection(); return true; }
            Key::Character(c) if c == "v" => { self.paste(); return true; }
            Key::Character(c) if c == "a" => { self.select_all(); return true; }
            Key::Character(c) if c == "x" => { self.cut_selection(); return true; }
            _ => {}
        }
    }
    // ... resto do handle_key_event
}
```

### 6.2 Problema: modifiers no widget

Atualmente `ModifiersState` e armazenado no `State` (render crate), mas nao
e passado para widgets. Opcoes:

**Opcao A**: Adicionar `modifiers: ModifiersState` ao `EventContext`
**Opcao B**: Adicionar `modifiers: ModifiersState` como parametro de `handle_key_event`
**Opcao C**: Usar `event.modifiers` (disponivel em winit 0.29 via `KeyEvent`)

Recomendacao: **Opcao B** — manter a interface limpa:
```rust
fn handle_key_event(&mut self, event: &KeyEvent, modifiers: ModifiersState) -> bool
```

### Estimativa de complexidade: Baixa (~50-80 linhas)

---

## 7. M11 - Terminal Widget

### Pre-requisitos
- [x] M8: Keyboard dispatch (widget recebe teclas)
- [x] M9: TextInput (padrao de input de texto)
- [x] M10: Clipboard (Ctrl+C/V)
- [x] Icon Fonts (prompt bonito)
- [x] Scroll por widget (scroll do output)

### Arquitetura

```
TerminalWidget
  |
  |-- pty_master: PtyMaster        (comunicacao com o shell)
  |-- reader_thread: JoinHandle    (le output do PTY em background)
  |-- output_lines: Signal<Vec<TermLine>>  (buffer de linhas renderizadas)
  |-- input_line: String           (linha sendo digitada)
  |-- cursor_pos: usize
  |-- vt_parser: vte::Parser       (interpreta codigos ANSI/VT100)
```

### 7.1 Dependencias

```toml
[dependencies]
portable-pty = "0.9"   # Pseudo-terminal (Windows/Linux/Mac)
vte = "0.13"           # Parser VT100/ANSI (padrao da industria, usado pelo Alacritty)
```

**Por que `vte` e nao parse manual?**
- Mesma crate usada pelo Alacritty (terminal GPU mais popular em Rust)
- Suporta todos os escape codes: cores, cursor movement, clear screen, etc.
- Bem mantida, ~0 dependencias

**Por que `portable-pty` e nao `std::process::Command`?**
- PTY e necessario para programas interativos (vim, htop, ssh)
- `Command` so funciona para stdin/stdout simples
- `portable-pty` abstrai ConPTY (Windows) e forkpty (Unix)

### 7.2 Grid de caracteres

Um terminal nao e texto livre — e uma **grade** (cols x rows) de celulas:

```rust
struct TermCell {
    character: char,
    fg_color: [u8; 3],
    bg_color: Option<[u8; 3]>,
    bold: bool,
}

struct TermGrid {
    cells: Vec<Vec<TermCell>>,  // [row][col]
    cols: usize,
    rows: usize,
    cursor_row: usize,
    cursor_col: usize,
}
```

O parser VT100 popula esta grid via callbacks do `vte::Perform`:

```rust
impl vte::Perform for TermGrid {
    fn print(&mut self, c: char) {
        self.cells[self.cursor_row][self.cursor_col] = TermCell { character: c, .. };
        self.cursor_col += 1;
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x0A => { self.cursor_row += 1; }          // \n
            0x0D => { self.cursor_col = 0; }           // \r
            0x08 => { self.cursor_col = self.cursor_col.saturating_sub(1); } // BS
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        // Cores ANSI: SGR (action = 'm')
        // Mover cursor: CUP (action = 'H'), CUU/CUD/CUF/CUB
        // Limpar tela: ED (action = 'J')
        // etc.
    }
}
```

### 7.3 Renderizacao

Cada celula e renderizada individualmente com fonte monoespacada:

```
Para cada row visivel:
  Para cada col:
    Se cell.bg_color != None:
      fill_rect(col_x, row_y, cell_w, cell_h, bg_color)
    draw_text_with_font(cell.character, pos, fg_color, monospace_font)
```

Usar a Nerd Font Mono (`SymbolsNerdFontMono-Regular.ttf`) ja incluida nos
assets para que caracteres de box-drawing e ícones renderizem corretamente.

Para performance, agrupar texto da mesma cor/linha em um unico `draw_text`
ao inves de um por celula.

### 7.4 Threading

```
+-------------------+          +-------------------+
|   UI Thread       |          |  Reader Thread    |
|                   |          |                   |
|  handle_key_event |--write-->|  pty_master       |
|                   |          |  (stdin do shell)  |
|                   |          |                   |
|  draw() <---------|--read----|  pty_master        |
|  (le output_buf)  |  (mutex) |  (stdout do shell) |
+-------------------+          +-------------------+
```

A reader thread le bytes do PTY, alimenta o `vte::Parser`, e atualiza o `TermGrid`
protegido por `Arc<Mutex<TermGrid>>`. A UI thread faz lock apenas no `draw()`.

### 7.5 Input de teclas

```rust
fn handle_key_event(&mut self, event: &KeyEvent, modifiers: ModifiersState) -> bool {
    // Ctrl+C -> envia SIGINT (byte 0x03)
    if modifiers.control_key() {
        if let Key::Character(c) = &event.logical_key {
            let byte = c.as_bytes()[0];
            // Ctrl+A=0x01, Ctrl+B=0x02, ..., Ctrl+Z=0x1A
            if byte >= b'a' && byte <= b'z' {
                let ctrl_byte = byte - b'a' + 1;
                self.pty_write(&[ctrl_byte]);
                return true;
            }
        }
    }

    // Teclas especiais -> escape sequences
    match &event.logical_key {
        Key::Named(NamedKey::Enter) => self.pty_write(b"\r"),
        Key::Named(NamedKey::Backspace) => self.pty_write(b"\x7f"),
        Key::Named(NamedKey::ArrowUp) => self.pty_write(b"\x1b[A"),
        Key::Named(NamedKey::ArrowDown) => self.pty_write(b"\x1b[B"),
        Key::Named(NamedKey::ArrowRight) => self.pty_write(b"\x1b[C"),
        Key::Named(NamedKey::ArrowLeft) => self.pty_write(b"\x1b[D"),
        Key::Character(c) => self.pty_write(c.as_bytes()),
        _ => return false,
    }
    true
}
```

### 7.6 API publica

```rust
let terminal = Terminal::new()
    .with_shell("cmd.exe")     // ou "bash", ou auto-detect
    .with_size(80, 24)         // colunas x linhas
    .with_font_size(14.0)
    .with_colors([0, 255, 0], [15, 15, 15]);  // fg, bg

WidgetNode::new(terminal, vec![])
```

### 7.7 Mudancas necessarias

| Arquivo                                  | Mudanca                          |
|------------------------------------------|----------------------------------|
| `bexa-ui-core/Cargo.toml`               | Adicionar `vte`, `portable-pty`  |
| `bexa-ui-core/src/widgets/terminal.rs`   | Novo widget                      |
| `bexa-ui-core/src/widgets/mod.rs`        | Registrar Terminal               |
| `bexa-ui-core/src/lib.rs`               | Re-exportar Terminal             |
| `bexa-ui/src/lib.rs`                    | Adicionar ao prelude             |
| `examples/terminal.rs`                   | Exemplo dedicado                 |
| `examples/Cargo.toml`                    | Registrar exemplo                |

### Estimativa de complexidade: Alta (~500-800 linhas)

---

## 8. Cronograma e Dependencias

```
Semana 1:  M8  - Keyboard Dispatch         [~50 linhas]
           M9  - TextInput Widget           [~250 linhas]
                  (ja testavel no dashboard)

Semana 2:  M10 - Clipboard                 [~60 linhas]
                  (TextInput fica completo)

Semana 3:  M11 - Terminal Widget            [~600 linhas]
           +    - PTY + VT100 parser
           +    - Grid de caracteres
           +    - Exemplo terminal.rs
```

### Grafo de dependencias

```
                    +---> M10 (Clipboard)
                    |         |
M8 (Keyboard) --+--+---> M9 (TextInput) --+--> M11 (Terminal)
                                           |
                              Scroll ------+
                              Icons -------+
                              Clipping ----+
```

---

## 9. Riscos e Mitigacoes

| Risco                                  | Mitigacao                                    |
|----------------------------------------|----------------------------------------------|
| `portable-pty` API mudou               | Verificar versao atual antes de implementar  |
| VT100 parsing e complexo               | Usar `vte` crate (battle-tested, Alacritty)  |
| Performance do terminal (muitas celulas)| Agrupar texto por linha, dirty-tracking      |
| Cursor blink requer redraw continuo    | `request_redraw` apenas quando input focado  |
| Clipboard nao funciona em Wayland      | `arboard` suporta Wayland via wl-clipboard   |
| Medida de texto imprecisa (v1)         | Usar monospace; refinar com glyphon metrics  |

---

## 10. Fora de Escopo (futuro)

- IME (Input Method Editor) para CJK
- Selecao de texto com mouse no TextInput
- Selecao de texto no terminal
- Multiplexacao de terminais (tabs/splits)
- SSH integrado
- Autocomplete
- Syntax highlighting

---

## 11. Decisoes em Aberto

1. **Monospace font**: Usar a Nerd Font Mono ja inclusa, ou embutir tambem uma
   fonte monoespacada para texto geral (ex: JetBrains Mono)?

2. **Terminal no core ou crate separada?**: `portable-pty` e `vte` sao deps
   pesadas. Podemos criar `bexa-ui-terminal` como crate opcional.

3. **Redraw strategy**: Quando o terminal tem output novo, como forcar redraw?
   Opcao: reader thread chama `window.request_redraw()` via channel/callback.

---

## 12. Referencias

- [vte crate](https://crates.io/crates/vte) - Parser VT100 (usado pelo Alacritty)
- [portable-pty](https://crates.io/crates/portable-pty) - Cross-platform PTY
- [arboard](https://crates.io/crates/arboard) - Cross-platform clipboard
- [Alacritty](https://github.com/alacritty/alacritty) - Terminal GPU reference
- [winit KeyEvent](https://docs.rs/winit/0.29/winit/event/struct.KeyEvent.html)
