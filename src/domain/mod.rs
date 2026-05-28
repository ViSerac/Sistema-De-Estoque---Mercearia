use chrono::NaiveDateTime;

#[derive(Clone, Debug)]
pub struct Categoria {
    pub id: i64,
    pub nome: String,
}

#[derive(Clone, Debug)]
pub struct Produto {
    pub id: i64,
    pub nome: String,
    pub codigo_de_barras: String,
    pub categoria_id: i64,
    pub categoria_nome: String,
    pub preco_custo: f64,
    pub preco_de_venda: f64,
    pub estoque_minimo: i64,
    pub quantidade_atual: i64,
}

impl Produto {
    pub fn estoque_baixo(&self) -> bool {
        self.quantidade_atual <= self.estoque_minimo
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PerfilUsuario {
    Dona,
    Funcionario,
}

impl PerfilUsuario {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Dona" => Self::Dona,
            _ => Self::Funcionario,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dona => "Dona",
            Self::Funcionario => "Funcionário",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Usuario {
    pub id: i64,
    pub nome: String,
    pub login: String,
    pub senha_hash: String,
    pub perfil: PerfilUsuario,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TipoMovimentacao {
    Entrada,
    Saida,
}

impl TipoMovimentacao {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Entrada" => Self::Entrada,
            _ => Self::Saida,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Entrada => "Entrada",
            Self::Saida => "Saída",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Movimentacao {
    pub id: i64,
    pub tipo: TipoMovimentacao,
    pub quantidade: i64,
    pub data_hora: NaiveDateTime,
    pub motivo: String,
    pub produto_id: i64,
    pub produto_nome: String,
    pub usuario_id: i64,
    pub usuario_nome: String,
}
