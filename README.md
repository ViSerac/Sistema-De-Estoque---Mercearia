# Sistema de Estoque — Mercearia

Aplicação desktop de gestão de estoque para mercearias, desenvolvida em Rust com interface gráfica nativa via [egui](https://github.com/emilk/egui).

## Funcionalidades

- **Login** com autenticação por senha (SHA-256) e controle de perfil
- **Dashboard** com cards de resumo, alertas de estoque baixo e movimentações recentes
- **Produtos** — cadastro, edição, exclusão e busca, com validação de preços e código de barras único
- **Categorias** — gerenciamento inline com proteção de integridade referencial
- **Movimentações** — registro de entradas e saídas com histórico completo
- **Relatórios** — estoque abaixo do mínimo e resumo mensal (perfil Dona)
- **Controle de acesso** por perfil: *Dona* (acesso total) e *Funcionário* (acesso restrito)

## Tecnologias

| Dependência | Versão | Uso |
|---|---|---|
| [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) | 0.31 | Janela nativa |
| [egui](https://github.com/emilk/egui) | 0.31 | Interface gráfica |
| [egui_extras](https://github.com/emilk/egui/tree/master/crates/egui_extras) | 0.31 | Tabelas |
| [rusqlite](https://github.com/rusqlite/rusqlite) | 0.40 | Banco de dados SQLite (bundled) |
| [chrono](https://github.com/chronotope/chrono) | 0.4 | Datas e horas |
| [sha2](https://github.com/RustCrypto/hashes) | 0.10 | Hash de senhas |

## Como executar

```bash
git clone https://github.com/ViSerac/Sistema-De-Estoque---Mercearia.git
cd Sistema-De-Estoque---Mercearia
cargo run --release
```

> Requer [Rust](https://rustup.rs/) instalado (edição 2024+).

O banco de dados `mercado.db` é criado automaticamente na mesma pasta do executável na primeira execução, já populado com categorias e produtos de exemplo.

## Acesso padrão

| Campo | Valor |
|---|---|
| Login | `admin` |
| Senha | `admin123` |
| Perfil | Dona (acesso total) |

## Estrutura do projeto

```
src/
├── domain/        # Structs de domínio (Produto, Categoria, Movimentacao, Usuario)
├── repository/    # Acesso ao banco de dados (SQLite via rusqlite)
├── service/       # Regras de negócio (autenticação, validação de estoque)
├── ui/            # Telas e componentes egui
│   ├── dashboard.rs
│   ├── produtos.rs
│   ├── categorias.rs
│   ├── movimentacoes.rs
│   └── relatorios.rs
├── util.rs        # Hash de senha
└── main.rs        # Ponto de entrada eframe
```

## Licença

MIT — veja [LICENSE](LICENSE).
