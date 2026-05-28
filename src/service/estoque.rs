use std::fmt;

use chrono::Local;
use rusqlite::Connection;

use crate::domain::{Movimentacao, Produto, TipoMovimentacao};
use crate::repository::{movimentacao, produto};

#[derive(Debug)]
pub enum ErroEstoque {
    PrecoCustoInvalido,
    PrecoVendaMenorQueCusto,
    EstoqueMinimoNegativo,
    BarcodeDuplicado,
    EstoqueInsuficiente { disponivel: i64 },
    QuantidadeZero,
    ProdutoNaoEncontrado,
    DbError(rusqlite::Error),
}

impl fmt::Display for ErroEstoque {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrecoCustoInvalido => write!(f, "Preço de custo deve ser maior que zero"),
            Self::PrecoVendaMenorQueCusto => {
                write!(f, "Preço de venda deve ser maior que o preço de custo")
            }
            Self::EstoqueMinimoNegativo => {
                write!(f, "Estoque mínimo não pode ser negativo")
            }
            Self::BarcodeDuplicado => write!(f, "Código de barras já cadastrado"),
            Self::EstoqueInsuficiente { disponivel } => {
                write!(f, "Estoque insuficiente (disponível: {})", disponivel)
            }
            Self::QuantidadeZero => write!(f, "Quantidade deve ser maior que zero"),
            Self::ProdutoNaoEncontrado => write!(f, "Produto não encontrado"),
            Self::DbError(e) => write!(f, "Erro de banco de dados: {}", e),
        }
    }
}

impl std::error::Error for ErroEstoque {}

impl From<rusqlite::Error> for ErroEstoque {
    fn from(e: rusqlite::Error) -> Self {
        Self::DbError(e)
    }
}

pub fn validar_produto(conn: &Connection, p: &Produto) -> Result<(), ErroEstoque> {
    if p.preco_custo <= 0.0 {
        return Err(ErroEstoque::PrecoCustoInvalido);
    }
    if p.preco_de_venda <= p.preco_custo {
        return Err(ErroEstoque::PrecoVendaMenorQueCusto);
    }
    if p.estoque_minimo < 0 {
        return Err(ErroEstoque::EstoqueMinimoNegativo);
    }
    if let Some(existente) = produto::buscar_por_barcode(conn, &p.codigo_de_barras)? {
        if existente.id != p.id {
            return Err(ErroEstoque::BarcodeDuplicado);
        }
    }
    Ok(())
}

pub fn registrar_entrada(
    conn: &Connection,
    produto_id: i64,
    quantidade: i64,
    motivo: &str,
    usuario_id: i64,
    usuario_nome: &str,
) -> Result<(), ErroEstoque> {
    if quantidade <= 0 {
        return Err(ErroEstoque::QuantidadeZero);
    }
    let p = produto::buscar_por_id(conn, produto_id)?
        .ok_or(ErroEstoque::ProdutoNaoEncontrado)?;
    let nova_qtd = p.quantidade_atual + quantidade;
    produto::atualizar_quantidade(conn, produto_id, nova_qtd)?;
    let mov = Movimentacao {
        id: 0,
        tipo: TipoMovimentacao::Entrada,
        quantidade,
        data_hora: Local::now().naive_local(),
        motivo: motivo.to_string(),
        produto_id,
        produto_nome: p.nome,
        usuario_id,
        usuario_nome: usuario_nome.to_string(),
    };
    movimentacao::inserir(conn, &mov)?;
    Ok(())
}

pub fn registrar_saida(
    conn: &Connection,
    produto_id: i64,
    quantidade: i64,
    motivo: &str,
    usuario_id: i64,
    usuario_nome: &str,
) -> Result<(), ErroEstoque> {
    if quantidade <= 0 {
        return Err(ErroEstoque::QuantidadeZero);
    }
    let p = produto::buscar_por_id(conn, produto_id)?
        .ok_or(ErroEstoque::ProdutoNaoEncontrado)?;
    if p.quantidade_atual < quantidade {
        return Err(ErroEstoque::EstoqueInsuficiente {
            disponivel: p.quantidade_atual,
        });
    }
    let nova_qtd = p.quantidade_atual - quantidade;
    produto::atualizar_quantidade(conn, produto_id, nova_qtd)?;
    let mov = Movimentacao {
        id: 0,
        tipo: TipoMovimentacao::Saida,
        quantidade,
        data_hora: Local::now().naive_local(),
        motivo: motivo.to_string(),
        produto_id,
        produto_nome: p.nome,
        usuario_id,
        usuario_nome: usuario_nome.to_string(),
    };
    movimentacao::inserir(conn, &mov)?;
    Ok(())
}
