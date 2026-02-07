O que falta (por prioridade)
1. Widgets essenciais
Checkbox / Radio / Toggle — qualquer form precisa
Select / Dropdown — combo box
Slider — range input
ScrollView — scroll genérico visível (scrollbar)
Image — exibir imagens/texturas
Table / DataGrid — dados tabulares
2. Layout & Estilo
Border radius nos containers (já temos SDF, falta expor na API)
Margin/gap mais ergonômico
Overflow scroll visível com scrollbar
Z-index / overlay — para dropdowns, modals, tooltips
Animações — transições básicas (hover, fade, slide)
3. Infraestrutura de projeto
Documentação — doc.rs com exemplos, guia de getting started
README decente no GitHub com screenshots/GIFs
Publicar no crates.io — cargo add bexa-ui precisa funcionar
CI/CD — GitHub Actions com build + clippy + tests
Testes — pelo menos unit tests dos widgets e layout
Licença — MIT ou Apache-2.0
4. Ergonomia do desenvolvedor
Reatividade — o sistema de Signal precisa ser robusto (rerender automático quando estado muda)
Event bubbling — onClick, onFocus, onHover propagáveis
Hot reload ou rebuild rápido
Acessibilidade — focus ring, tab order, screen reader (longo prazo)
5. Diferencial competitivo
A comunidade Rust já tem egui, iced, slint, dioxus. Para se destacar, o BexaUI precisa de um nicho claro:

Terminal widget embutido (M11) — nenhum toolkit Rust tem isso bem feito
Foco em aplicações de infraestrutura/devtools — dashboard, monitoring, admin panels
Simplicidade — menos boilerplate que iced, mais nativo que egui