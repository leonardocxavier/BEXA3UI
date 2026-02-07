Roadmap (organizado por estado e prioridade)

Feito
- Widgets: Checkbox, Radio, Select/Dropdown, Table/DataGrid, Tabs, TreeView, BarChart, Tooltip, Modal
- Layout/estilo: border radius em Container, padding/gap, overlay para dropdown/tooltip/modal
- Scroll: Container com scroll e scrollbar overlay (drag + wheel)
- Exemplos: dashboards e telas de controle
- Diferencial: Terminal widget inicial e foco em dashboards/devtools

Em andamento / precisa consolidar
- Reatividade: Signals com redraw continuo (precisa otimizar para redraw sob demanda)
- Event bubbling: consumo por widget e foco ok, falta propagar hover/click de forma mais refinada

Falta (prioridade alta)
1. Widgets essenciais restantes
Toggle — switch simples
Slider — range input
ScrollView generico com scrollbar visivel configuravel
Image — exibir imagens/texturas

2. Layout & Estilo
Margin/gap mais ergonomico (helpers e defaults melhores)
Animações — transicoes basicas (hover, fade, slide)

3. Infraestrutura de projeto
Documentacao — doc.rs com exemplos, guia de getting started
README com screenshots/GIFs
Publicar no crates.io — cargo add bexa-ui
CI/CD — GitHub Actions (build + clippy + tests)
Testes — unit tests de widgets/layout
Licenca — MIT ou Apache-2.0

4. Ergonomia do desenvolvedor
Hot reload ou rebuild rapido
Acessibilidade — focus ring, tab order, screen reader (longo prazo)

5. Diferencial competitivo (reforcar)
Terminal widget embutido (M11)
Foco em apps de infraestrutura/devtools (dashboard, monitoring, admin panels)
Simplicidade — menos boilerplate que iced, mais nativo que egui