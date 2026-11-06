use macroquad::prelude::*;

use crate::particula::Particula;

pub struct GradeEspacial {
    tamanho_celula: f32,
    origem: Vec2,
    colunas: usize,
    linhas: usize,
    cabecas: Vec<i32>,
    proximos: Vec<i32>,
}

impl GradeEspacial {
    pub fn nova(tamanho_celula: f32) -> Self {
        Self {
            tamanho_celula: tamanho_celula.max(8.0),
            origem: Vec2::ZERO,
            colunas: 1,
            linhas: 1,
            cabecas: Vec::new(),
            proximos: Vec::new(),
        }
    }

    pub fn reconstruir(&mut self, limites: Rect, particulas: &[Particula]) {
        self.origem = vec2(limites.x, limites.y);
        self.colunas = ((limites.w.max(1.0) / self.tamanho_celula).ceil() as usize) + 2;
        self.linhas = ((limites.h.max(1.0) / self.tamanho_celula).ceil() as usize) + 2;

        let total_celulas = self.colunas * self.linhas;
        if self.cabecas.len() < total_celulas {
            self.cabecas.resize(total_celulas, -1);
        }
        self.cabecas[..total_celulas].fill(-1);

        if self.proximos.len() < particulas.len() {
            self.proximos.resize(particulas.len(), -1);
        }

        for (indice, particula) in particulas.iter().enumerate() {
            let (coluna, linha) = self.coordenadas_celula(particula.posicao);
            let indice_celula = linha * self.colunas + coluna;
            self.proximos[indice] = self.cabecas[indice_celula];
            self.cabecas[indice_celula] = indice as i32;
        }
    }

    pub fn coordenadas_celula(&self, posicao: Vec2) -> (usize, usize) {
        let relativo_x = ((posicao.x - self.origem.x) / self.tamanho_celula).floor() as isize + 1;
        let relativo_y = ((posicao.y - self.origem.y) / self.tamanho_celula).floor() as isize + 1;
        let coluna = relativo_x.clamp(0, (self.colunas as isize) - 1) as usize;
        let linha = relativo_y.clamp(0, (self.linhas as isize) - 1) as usize;
        (coluna, linha)
    }

    pub fn colunas(&self) -> usize {
        self.colunas
    }

    pub fn linhas(&self) -> usize {
        self.linhas
    }

    pub fn cabeca_em(&self, coluna: usize, linha: usize) -> i32 {
        self.cabecas[linha * self.colunas + coluna]
    }

    pub fn proximo_de(&self, indice: usize) -> i32 {
        self.proximos[indice]
    }
}
