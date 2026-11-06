use macroquad::prelude::*;

use crate::ambiente::{AncoraForca, Emissor, ModoForca, ModoGravidade, ModoVisual};
use crate::fisica::MundoSimulacao;
use crate::materiais::MaterialParticula;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cena {
    Livre,
    Ampulheta,
    Cascata,
    Orbita,
    Alquimia,
}

impl Cena {
    pub fn proxima(self) -> Self {
        match self {
            Cena::Livre => Cena::Ampulheta,
            Cena::Ampulheta => Cena::Cascata,
            Cena::Cascata => Cena::Orbita,
            Cena::Orbita => Cena::Alquimia,
            Cena::Alquimia => Cena::Livre,
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Cena::Livre => "Livre",
            Cena::Ampulheta => "Ampulheta",
            Cena::Cascata => "Cascata",
            Cena::Orbita => "Órbita",
            Cena::Alquimia => "Alquimia",
        }
    }

    pub fn aplicar(self, mundo: &mut MundoSimulacao, limites: Rect) {
        mundo.limpar_tudo();
        mundo.pausado = false;

        let inicio_x = limites.x;
        let inicio_y = limites.y;
        let largura = limites.w.max(320.0);
        let altura = limites.h.max(240.0);
        let centro = vec2(inicio_x + largura * 0.5, inicio_y + altura * 0.5);

        match self {
            Cena::Livre => {
                mundo.modo_gravidade = ModoGravidade::Baixo;
                mundo.modo_visual = ModoVisual::Material;

                mundo.adicionar_obstaculo(
                    vec2(inicio_x + largura * 0.18, inicio_y + altura * 0.44),
                    vec2(inicio_x + largura * 0.38, inicio_y + altura * 0.54),
                    5.5,
                );
                mundo.adicionar_obstaculo(
                    vec2(inicio_x + largura * 0.82, inicio_y + altura * 0.44),
                    vec2(inicio_x + largura * 0.62, inicio_y + altura * 0.54),
                    5.5,
                );

                for indice in 0..220 {
                    let posicao = vec2(
                        rand::gen_range(inicio_x + 40.0, inicio_x + largura - 40.0),
                        rand::gen_range(inicio_y + 30.0, inicio_y + altura * 0.42),
                    );
                    let velocidade = vec2(rand::gen_range(-160.0, 160.0), rand::gen_range(-40.0, 40.0));
                    let material = if indice % 3 == 0 {
                        MaterialParticula::Agua
                    } else {
                        MaterialParticula::Elastico
                    };
                    mundo.adicionar_particula(posicao, velocidade, material);
                }
            }
            Cena::Ampulheta => {
                mundo.modo_gravidade = ModoGravidade::Baixo;
                mundo.modo_visual = ModoVisual::Material;

                let abertura = 19.0;
                let largura_topo = (largura * 0.22).clamp(130.0, 240.0);
                let topo_y = inicio_y + altura * 0.12;
                let meio_y = inicio_y + altura * 0.50;
                let base_y = inicio_y + altura * 0.88;

                mundo.adicionar_obstaculo(
                    vec2(centro.x - largura_topo, topo_y),
                    vec2(centro.x - abertura, meio_y),
                    6.0,
                );
                mundo.adicionar_obstaculo(
                    vec2(centro.x + largura_topo, topo_y),
                    vec2(centro.x + abertura, meio_y),
                    6.0,
                );
                mundo.adicionar_obstaculo(
                    vec2(centro.x - abertura, meio_y),
                    vec2(centro.x - largura_topo, base_y),
                    6.0,
                );
                mundo.adicionar_obstaculo(
                    vec2(centro.x + abertura, meio_y),
                    vec2(centro.x + largura_topo, base_y),
                    6.0,
                );
                mundo.adicionar_obstaculo(
                    vec2(centro.x - largura_topo, base_y),
                    vec2(centro.x + largura_topo, base_y),
                    6.5,
                );

                for _ in 0..520 {
                    let proporcao_altura = rand::gen_range(0.08f32, 0.86f32);
                    let posicao_y = topo_y + (meio_y - topo_y - 24.0) * proporcao_altura;
                    let meia_largura_disponivel =
                        (largura_topo * (1.0 - proporcao_altura) + abertura * proporcao_altura - 14.0).max(8.0);
                    let posicao_x = centro.x + rand::gen_range(-meia_largura_disponivel, meia_largura_disponivel);
                    mundo.adicionar_particula(vec2(posicao_x, posicao_y), Vec2::ZERO, MaterialParticula::Areia);
                }
            }
            Cena::Cascata => {
                mundo.modo_gravidade = ModoGravidade::Baixo;
                mundo.modo_visual = ModoVisual::Material;

                mundo.adicionar_obstaculo(
                    vec2(inicio_x + largura * 0.10, inicio_y + altura * 0.22),
                    vec2(inicio_x + largura * 0.46, inicio_y + altura * 0.36),
                    6.0,
                );
                mundo.adicionar_obstaculo(
                    vec2(inicio_x + largura * 0.78, inicio_y + altura * 0.42),
                    vec2(inicio_x + largura * 0.36, inicio_y + altura * 0.58),
                    6.0,
                );
                mundo.adicionar_obstaculo(
                    vec2(inicio_x + largura * 0.20, inicio_y + altura * 0.64),
                    vec2(inicio_x + largura * 0.58, inicio_y + altura * 0.76),
                    6.0,
                );

                mundo.emissores.push(Emissor {
                    posicao: vec2(inicio_x + largura * 0.15, inicio_y + altura * 0.14),
                    direcao: vec2(0.55, 0.83).normalize(),
                    material: MaterialParticula::Agua,
                    intervalo: 0.014,
                    acumulador: 0.0,
                    velocidade_inicial: 145.0,
                });

                for _ in 0..180 {
                    let posicao_x = rand::gen_range(inicio_x + largura * 0.30, inicio_x + largura * 0.88);
                    let posicao_y = rand::gen_range(inicio_y + altura * 0.84, inicio_y + altura - 16.0);
                    mundo.adicionar_particula(vec2(posicao_x, posicao_y), Vec2::ZERO, MaterialParticula::Areia);
                }
            }
            Cena::Orbita => {
                mundo.modo_gravidade = ModoGravidade::Zero;
                mundo.modo_visual = ModoVisual::Rastro;

                let raio_orbita_maximo = largura.min(altura) * 0.42;
                mundo.ancoras.push(AncoraForca {
                    posicao: centro,
                    modo: ModoForca::Atrair,
                    raio_acao: (raio_orbita_maximo * 1.18).max(220.0),
                    fase_animacao: 0.0,
                });

                for indice in 0..400 {
                    let angulo = rand::gen_range(0.0, std::f32::consts::TAU);
                    let distancia = rand::gen_range(46.0, raio_orbita_maximo.max(90.0));
                    let direcao_radial = vec2(angulo.cos(), angulo.sin());
                    let posicao = centro + direcao_radial * distancia;
                    let tangente = vec2(-direcao_radial.y, direcao_radial.x);
                    let decaimento = (1.0 - distancia / (raio_orbita_maximo * 1.18).max(220.0)).max(0.12);
                    let rapidez_orbital = (1150.0 * decaimento * distancia).sqrt() * 0.92;
                    let material = if indice % 4 == 0 {
                        MaterialParticula::Magnetico
                    } else {
                        MaterialParticula::Elastico
                    };
                    mundo.adicionar_particula(posicao, tangente * rapidez_orbital, material);
                }
            }
            Cena::Alquimia => {
                mundo.modo_gravidade = ModoGravidade::Baixo;
                mundo.modo_visual = ModoVisual::Material;

                mundo.adicionar_obstaculo(
                    vec2(inicio_x + largura * 0.25, inicio_y + altura * 0.20),
                    vec2(centro.x - 16.0, inicio_y + altura * 0.30),
                    5.5,
                );
                mundo.adicionar_obstaculo(
                    vec2(inicio_x + largura * 0.75, inicio_y + altura * 0.20),
                    vec2(centro.x + 16.0, inicio_y + altura * 0.30),
                    5.5,
                );

                mundo.adicionar_obstaculo(
                    vec2(centro.x - 150.0, inicio_y + altura * 0.62),
                    vec2(centro.x - 95.0, inicio_y + altura * 0.76),
                    6.0,
                );
                mundo.adicionar_obstaculo(
                    vec2(centro.x - 95.0, inicio_y + altura * 0.76),
                    vec2(centro.x + 95.0, inicio_y + altura * 0.76),
                    6.0,
                );
                mundo.adicionar_obstaculo(
                    vec2(centro.x + 95.0, inicio_y + altura * 0.76),
                    vec2(centro.x + 150.0, inicio_y + altura * 0.62),
                    6.0,
                );

                for _ in 0..190 {
                    let posicao_x = rand::gen_range(centro.x - 90.0, centro.x + 90.0);
                    let posicao_y = rand::gen_range(inicio_y + altura * 0.64, inicio_y + altura * 0.74);
                    mundo.adicionar_particula(vec2(posicao_x, posicao_y), Vec2::ZERO, MaterialParticula::Agua);
                }

                mundo.emissores.push(Emissor {
                    posicao: vec2(centro.x, inicio_y + altura * 0.84),
                    direcao: vec2(0.0, -1.0),
                    material: MaterialParticula::Fogo,
                    intervalo: 0.020,
                    acumulador: 0.0,
                    velocidade_inicial: 155.0,
                });

                for _ in 0..70 {
                    let posicao_x = rand::gen_range(inicio_x + largura * 0.16, inicio_x + largura * 0.84);
                    let posicao_y = rand::gen_range(inicio_y + altura * 0.36, inicio_y + altura * 0.54);
                    mundo.adicionar_particula(
                        vec2(posicao_x, posicao_y),
                        vec2(rand::gen_range(-50.0, 50.0), rand::gen_range(-30.0, 30.0)),
                        MaterialParticula::Magnetico,
                    );
                }
            }
        }
    }
}
