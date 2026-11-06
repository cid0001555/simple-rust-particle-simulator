use macroquad::prelude::*;

use crate::materiais::{interpolar_cor, MaterialParticula};

#[derive(Clone, Debug)]
pub struct Particula {
    pub posicao: Vec2,
    pub posicao_anterior: Vec2,
    pub velocidade: Vec2,
    pub raio: f32,
    pub raio_inicial: f32,
    pub massa_inversa: f32,
    pub cor_base: Color,
    pub material: MaterialParticula,
    pub sinal_carga: f32,
    pub vida_restante: f32,
    pub vida_total: f32,
    pub perecivel: bool,
    pub inflamavel: bool,
    pub condensavel: bool,
    pub tempo_condensacao: f32,
    pub semente_fase: f32,
}

impl Particula {
    pub fn nova(posicao: Vec2, velocidade: Vec2, material: MaterialParticula, alternador_carga: bool) -> Self {
        let propriedades = material.propriedades();
        let raio = rand::gen_range(propriedades.raio_minimo, propriedades.raio_maximo);
        let sinal_carga = if material == MaterialParticula::Magnetico {
            if alternador_carga { 1.0 } else { -1.0 }
        } else {
            0.0
        };
        let massa = (raio * raio * propriedades.densidade).max(0.5);
        let vida_total = if material == MaterialParticula::Fogo {
            rand::gen_range(1.1, 2.4)
        } else {
            1.0
        };

        Self {
            posicao,
            posicao_anterior: posicao,
            velocidade,
            raio,
            raio_inicial: raio,
            massa_inversa: 1.0 / massa,
            cor_base: material.gerar_cor_inicial(sinal_carga),
            material,
            sinal_carga,
            vida_restante: vida_total,
            vida_total,
            perecivel: material == MaterialParticula::Fogo,
            inflamavel: material == MaterialParticula::Gas,
            condensavel: false,
            tempo_condensacao: 0.0,
            semente_fase: rand::gen_range(0.0, std::f32::consts::TAU),
        }
    }

    pub fn converter_para_vapor(&mut self) {
        let propriedades = MaterialParticula::Gas.propriedades();
        let novo_raio = rand::gen_range(4.2, 5.8);
        let massa = (novo_raio * novo_raio * propriedades.densidade).max(0.5);
        self.material = MaterialParticula::Gas;
        self.raio = novo_raio;
        self.raio_inicial = novo_raio;
        self.massa_inversa = 1.0 / massa;
        self.cor_base = Color::new(0.80, 0.88, 0.98, 0.62);
        self.perecivel = false;
        self.inflamavel = false;
        self.condensavel = true;
        self.tempo_condensacao = rand::gen_range(4.2, 7.2);
        self.velocidade.y -= rand::gen_range(45.0, 110.0);
        self.velocidade.x += rand::gen_range(-40.0, 40.0);
    }

    pub fn condensar_para_agua(&mut self) {
        let propriedades = MaterialParticula::Agua.propriedades();
        let novo_raio = rand::gen_range(propriedades.raio_minimo, propriedades.raio_maximo);
        let massa = (novo_raio * novo_raio * propriedades.densidade).max(0.5);
        self.material = MaterialParticula::Agua;
        self.raio = novo_raio;
        self.raio_inicial = novo_raio;
        self.massa_inversa = 1.0 / massa;
        self.cor_base = MaterialParticula::Agua.gerar_cor_inicial(0.0);
        self.perecivel = false;
        self.inflamavel = false;
        self.condensavel = false;
        self.tempo_condensacao = 0.0;
    }

    pub fn inflamar(&mut self) {
        let propriedades = MaterialParticula::Fogo.propriedades();
        let novo_raio = rand::gen_range(propriedades.raio_minimo, propriedades.raio_maximo);
        let massa = (novo_raio * novo_raio * propriedades.densidade).max(0.5);
        let vida = rand::gen_range(0.9, 1.8);
        self.material = MaterialParticula::Fogo;
        self.raio = novo_raio;
        self.raio_inicial = novo_raio;
        self.massa_inversa = 1.0 / massa;
        self.cor_base = MaterialParticula::Fogo.gerar_cor_inicial(0.0);
        self.vida_total = vida;
        self.vida_restante = vida;
        self.perecivel = true;
        self.inflamavel = false;
        self.condensavel = false;
        let angulo = rand::gen_range(0.0, std::f32::consts::TAU);
        let impulso = rand::gen_range(60.0, 170.0);
        self.velocidade += vec2(angulo.cos(), angulo.sin()) * impulso + vec2(0.0, -70.0);
    }

    pub fn cor_renderizacao(&self) -> Color {
        match self.material {
            MaterialParticula::Fogo => {
                let fracao_vida = (self.vida_restante / self.vida_total.max(0.01)).clamp(0.0, 1.0);
                let nucleo_quente = Color::new(1.0, 0.96, 0.68, 0.98);
                let laranja_vivo = Color::new(1.0, 0.52, 0.12, 0.95);
                let brasa_escura = Color::new(0.76, 0.16, 0.08, 0.80);
                let fumaca = Color::new(0.28, 0.26, 0.30, 0.32);

                if fracao_vida > 0.68 {
                    interpolar_cor(laranja_vivo, nucleo_quente, (fracao_vida - 0.68) / 0.32)
                } else if fracao_vida > 0.30 {
                    interpolar_cor(brasa_escura, laranja_vivo, (fracao_vida - 0.30) / 0.38)
                } else {
                    interpolar_cor(fumaca, brasa_escura, fracao_vida / 0.30)
                }
            }
            MaterialParticula::Agua => {
                let rapidez = self.velocidade.length();
                let espuma = ((rapidez - 190.0) / 360.0).clamp(0.0, 0.68);
                interpolar_cor(self.cor_base, Color::new(0.84, 0.95, 1.0, 0.96), espuma)
            }
            _ => self.cor_base,
        }
    }
}
