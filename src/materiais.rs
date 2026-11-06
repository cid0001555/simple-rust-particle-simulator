use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MaterialParticula {
    Elastico,
    Agua,
    Areia,
    Gas,
    Fogo,
    Magnetico,
}

#[derive(Clone, Copy, Debug)]
pub struct PropriedadesMaterial {
    pub densidade: f32,
    pub restituicao: f32,
    pub atrito: f32,
    pub amortecimento_ar: f32,
    pub escala_gravidade: f32,
    pub raio_minimo: f32,
    pub raio_maximo: f32,
    pub emissivo: bool,
}

pub fn interpolar_cor(cor_inicial: Color, cor_final: Color, fator: f32) -> Color {
    let proporcao = fator.clamp(0.0, 1.0);
    Color::new(
        cor_inicial.r + (cor_final.r - cor_inicial.r) * proporcao,
        cor_inicial.g + (cor_final.g - cor_inicial.g) * proporcao,
        cor_inicial.b + (cor_final.b - cor_inicial.b) * proporcao,
        cor_inicial.a + (cor_final.a - cor_inicial.a) * proporcao,
    )
}

pub fn cor_por_velocidade(rapidez: f32) -> Color {
    let normalizada = (rapidez / 620.0).clamp(0.0, 1.0);
    let azul_profundo = Color::new(0.14, 0.24, 0.68, 1.0);
    let ciano_vivo = Color::new(0.08, 0.78, 0.92, 1.0);
    let verde_esmeralda = Color::new(0.22, 0.92, 0.56, 1.0);
    let ouro_quente = Color::new(1.00, 0.78, 0.18, 1.0);
    let coral_incandescente = Color::new(1.00, 0.34, 0.30, 1.0);

    if normalizada < 0.25 {
        interpolar_cor(azul_profundo, ciano_vivo, normalizada / 0.25)
    } else if normalizada < 0.50 {
        interpolar_cor(ciano_vivo, verde_esmeralda, (normalizada - 0.25) / 0.25)
    } else if normalizada < 0.78 {
        interpolar_cor(verde_esmeralda, ouro_quente, (normalizada - 0.50) / 0.28)
    } else {
        interpolar_cor(ouro_quente, coral_incandescente, (normalizada - 0.78) / 0.22)
    }
}

impl MaterialParticula {
    pub fn todos() -> [MaterialParticula; 6] {
        [
            MaterialParticula::Elastico,
            MaterialParticula::Agua,
            MaterialParticula::Areia,
            MaterialParticula::Gas,
            MaterialParticula::Fogo,
            MaterialParticula::Magnetico,
        ]
    }

    pub fn nome(self) -> &'static str {
        match self {
            MaterialParticula::Elastico => "Elástico",
            MaterialParticula::Agua => "Água",
            MaterialParticula::Areia => "Areia",
            MaterialParticula::Gas => "Gás",
            MaterialParticula::Fogo => "Fogo",
            MaterialParticula::Magnetico => "Ímã",
        }
    }

    pub fn atalho(self) -> &'static str {
        match self {
            MaterialParticula::Elastico => "1",
            MaterialParticula::Agua => "2",
            MaterialParticula::Areia => "3",
            MaterialParticula::Gas => "4",
            MaterialParticula::Fogo => "5",
            MaterialParticula::Magnetico => "6",
        }
    }

    pub fn cor_icone(self) -> Color {
        match self {
            MaterialParticula::Elastico => Color::new(0.96, 0.44, 0.58, 1.0),
            MaterialParticula::Agua => Color::new(0.20, 0.66, 0.98, 1.0),
            MaterialParticula::Areia => Color::new(0.94, 0.78, 0.36, 1.0),
            MaterialParticula::Gas => Color::new(0.68, 0.46, 0.94, 1.0),
            MaterialParticula::Fogo => Color::new(1.00, 0.50, 0.14, 1.0),
            MaterialParticula::Magnetico => Color::new(0.18, 0.94, 0.84, 1.0),
        }
    }

    pub fn propriedades(self) -> PropriedadesMaterial {
        match self {
            MaterialParticula::Elastico => PropriedadesMaterial {
                densidade: 1.25,
                restituicao: 0.88,
                atrito: 0.04,
                amortecimento_ar: 0.9994,
                escala_gravidade: 1.0,
                raio_minimo: 4.4,
                raio_maximo: 7.2,
                emissivo: false,
            },
            MaterialParticula::Agua => PropriedadesMaterial {
                densidade: 1.0,
                restituicao: 0.06,
                atrito: 0.012,
                amortecimento_ar: 0.9985,
                escala_gravidade: 1.0,
                raio_minimo: 3.8,
                raio_maximo: 4.6,
                emissivo: false,
            },
            MaterialParticula::Areia => PropriedadesMaterial {
                densidade: 2.45,
                restituicao: 0.02,
                atrito: 0.72,
                amortecimento_ar: 0.996,
                escala_gravidade: 1.15,
                raio_minimo: 3.1,
                raio_maximo: 4.4,
                emissivo: false,
            },
            MaterialParticula::Gas => PropriedadesMaterial {
                densidade: 0.24,
                restituicao: 0.60,
                atrito: 0.02,
                amortecimento_ar: 0.991,
                escala_gravidade: -0.42,
                raio_minimo: 4.6,
                raio_maximo: 6.8,
                emissivo: false,
            },
            MaterialParticula::Fogo => PropriedadesMaterial {
                densidade: 0.18,
                restituicao: 0.25,
                atrito: 0.04,
                amortecimento_ar: 0.986,
                escala_gravidade: -0.92,
                raio_minimo: 3.8,
                raio_maximo: 6.2,
                emissivo: true,
            },
            MaterialParticula::Magnetico => PropriedadesMaterial {
                densidade: 1.55,
                restituicao: 0.74,
                atrito: 0.06,
                amortecimento_ar: 0.997,
                escala_gravidade: 0.85,
                raio_minimo: 4.4,
                raio_maximo: 5.8,
                emissivo: true,
            },
        }
    }

    pub fn gerar_cor_inicial(self, sinal_carga: f32) -> Color {
        let variacao: f32 = rand::gen_range(-0.06, 0.06);
        match self {
            MaterialParticula::Elastico => {
                let paleta = [
                    Color::new(0.96, 0.38, 0.52, 1.0),
                    Color::new(0.98, 0.68, 0.24, 1.0),
                    Color::new(0.26, 0.88, 0.64, 1.0),
                    Color::new(0.32, 0.68, 0.98, 1.0),
                    Color::new(0.74, 0.48, 0.98, 1.0),
                ];
                let escolha = paleta[rand::gen_range(0, paleta.len())];
                Color::new(
                    (escolha.r + variacao).clamp(0.0, 1.0),
                    (escolha.g + variacao).clamp(0.0, 1.0),
                    (escolha.b + variacao).clamp(0.0, 1.0),
                    1.0,
                )
            }
            MaterialParticula::Agua => Color::new(
                (0.12 + variacao * 0.5).clamp(0.05, 0.35),
                (0.58 + variacao).clamp(0.42, 0.76),
                (0.96 + variacao * 0.4).clamp(0.82, 1.0),
                0.92,
            ),
            MaterialParticula::Areia => {
                let tom: f32 = rand::gen_range(-0.08, 0.08);
                Color::new(
                    (0.90 + tom).clamp(0.72, 1.0),
                    (0.74 + tom * 0.85).clamp(0.56, 0.90),
                    (0.36 + tom * 0.5).clamp(0.22, 0.52),
                    1.0,
                )
            }
            MaterialParticula::Gas => Color::new(
                (0.60 + variacao).clamp(0.42, 0.82),
                (0.42 + variacao).clamp(0.28, 0.64),
                (0.90 + variacao).clamp(0.74, 1.0),
                0.68,
            ),
            MaterialParticula::Fogo => Color::new(1.0, 0.86, 0.32, 0.96),
            MaterialParticula::Magnetico => {
                if sinal_carga >= 0.0 {
                    Color::new(
                        (0.98 + variacao * 0.3).clamp(0.85, 1.0),
                        (0.26 + variacao).clamp(0.14, 0.42),
                        (0.66 + variacao).clamp(0.50, 0.84),
                        1.0,
                    )
                } else {
                    Color::new(
                        (0.14 + variacao).clamp(0.05, 0.32),
                        (0.92 + variacao * 0.4).clamp(0.78, 1.0),
                        (0.88 + variacao * 0.4).clamp(0.74, 1.0),
                        1.0,
                    )
                }
            }
        }
    }
}
