use macroquad::prelude::*;

use crate::materiais::MaterialParticula;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModoGravidade {
    Baixo,
    Zero,
    Centro,
    Invertida,
}

impl ModoGravidade {
    pub fn proximo(self) -> Self {
        match self {
            ModoGravidade::Baixo => ModoGravidade::Zero,
            ModoGravidade::Zero => ModoGravidade::Centro,
            ModoGravidade::Centro => ModoGravidade::Invertida,
            ModoGravidade::Invertida => ModoGravidade::Baixo,
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            ModoGravidade::Baixo => "Normal",
            ModoGravidade::Zero => "Zero-G",
            ModoGravidade::Centro => "Central",
            ModoGravidade::Invertida => "Invertida",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModoForca {
    Repelir,
    Atrair,
    Vortice,
    Arrastar,
}

impl ModoForca {
    pub fn proximo(self) -> Self {
        match self {
            ModoForca::Repelir => ModoForca::Atrair,
            ModoForca::Atrair => ModoForca::Vortice,
            ModoForca::Vortice => ModoForca::Arrastar,
            ModoForca::Arrastar => ModoForca::Repelir,
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            ModoForca::Repelir => "Repelir",
            ModoForca::Atrair => "Atrair",
            ModoForca::Vortice => "Vórtice",
            ModoForca::Arrastar => "Arrastar",
        }
    }

    pub fn cor(self) -> Color {
        match self {
            ModoForca::Repelir => Color::new(0.98, 0.56, 0.24, 1.0),
            ModoForca::Atrair => Color::new(0.62, 0.44, 0.98, 1.0),
            ModoForca::Vortice => Color::new(0.20, 0.88, 0.86, 1.0),
            ModoForca::Arrastar => Color::new(0.46, 0.92, 0.48, 1.0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModoVisual {
    Material,
    Velocidade,
    Rastro,
}

impl ModoVisual {
    pub fn proximo(self) -> Self {
        match self {
            ModoVisual::Material => ModoVisual::Velocidade,
            ModoVisual::Velocidade => ModoVisual::Rastro,
            ModoVisual::Rastro => ModoVisual::Material,
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            ModoVisual::Material => "Material",
            ModoVisual::Velocidade => "Térmico",
            ModoVisual::Rastro => "Rastro",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FerramentaAtiva {
    Material(MaterialParticula),
    Parede,
    Ancora,
    Fonte,
    Borracha,
}

#[derive(Clone, Copy, Debug)]
pub struct ObstaculoSegmento {
    pub inicio: Vec2,
    pub fim: Vec2,
    pub espessura: f32,
}

impl ObstaculoSegmento {
    pub fn ponto_mais_proximo(&self, ponto: Vec2) -> Vec2 {
        let segmento = self.fim - self.inicio;
        let comprimento_quadrado = segmento.length_squared();
        if comprimento_quadrado <= 0.0001 {
            return self.inicio;
        }
        let projecao = ((ponto - self.inicio).dot(segmento) / comprimento_quadrado).clamp(0.0, 1.0);
        self.inicio + segmento * projecao
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AncoraForca {
    pub posicao: Vec2,
    pub modo: ModoForca,
    pub raio_acao: f32,
    pub fase_animacao: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Emissor {
    pub posicao: Vec2,
    pub direcao: Vec2,
    pub material: MaterialParticula,
    pub intervalo: f32,
    pub acumulador: f32,
    pub velocidade_inicial: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct OndaVisual {
    pub centro: Vec2,
    pub raio_atual: f32,
    pub raio_maximo: f32,
    pub vida_restante: f32,
    pub vida_total: f32,
    pub cor: Color,
}
