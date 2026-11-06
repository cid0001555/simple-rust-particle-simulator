use macroquad::prelude::*;

use crate::ambiente::{
    AncoraForca, Emissor, ModoForca, ModoGravidade, ModoVisual, ObstaculoSegmento, OndaVisual,
};
use crate::grade_espacial::GradeEspacial;
use crate::materiais::MaterialParticula;
use crate::particula::Particula;

pub const LIMITE_MAXIMO_PARTICULAS: usize = 4200;
const GRAVIDADE_PADRAO: f32 = 580.0;
const VELOCIDADE_MAXIMA: f32 = 960.0;
const SUBPASSOS_FISICA: usize = 3;
const TAMANHO_CELULA_GRADE: f32 = 22.0;
const RAIO_INTERACAO_FLUIDO: f32 = 19.5;
const RAIO_INTERACAO_MAGNETICA: f32 = 52.0;

pub struct MundoSimulacao {
    pub particulas: Vec<Particula>,
    pub obstaculos: Vec<ObstaculoSegmento>,
    pub ancoras: Vec<AncoraForca>,
    pub emissores: Vec<Emissor>,
    pub ondas: Vec<OndaVisual>,
    pub modo_gravidade: ModoGravidade,
    pub modo_forca: ModoForca,
    pub modo_visual: ModoVisual,
    pub pausado: bool,
    pub tempo_acumulado: f32,
    alternador_carga: bool,
    grade: GradeEspacial,
}

impl MundoSimulacao {
    pub fn novo() -> Self {
        Self {
            particulas: Vec::with_capacity(1024),
            obstaculos: Vec::new(),
            ancoras: Vec::new(),
            emissores: Vec::new(),
            ondas: Vec::new(),
            modo_gravidade: ModoGravidade::Baixo,
            modo_forca: ModoForca::Repelir,
            modo_visual: ModoVisual::Material,
            pausado: false,
            tempo_acumulado: 0.0,
            alternador_carga: false,
            grade: GradeEspacial::nova(TAMANHO_CELULA_GRADE),
        }
    }

    pub fn limpar_tudo(&mut self) {
        self.particulas.clear();
        self.obstaculos.clear();
        self.ancoras.clear();
        self.emissores.clear();
        self.ondas.clear();
    }

    pub fn limpar_particulas(&mut self) {
        self.particulas.clear();
    }

    pub fn adicionar_particula(&mut self, posicao: Vec2, velocidade: Vec2, material: MaterialParticula) {
        if self.particulas.len() >= LIMITE_MAXIMO_PARTICULAS {
            return;
        }
        self.alternador_carga = !self.alternador_carga;
        let nova = Particula::nova(posicao, velocidade, material, self.alternador_carga);
        self.particulas.push(nova);
    }

    pub fn criar_pincel(
        &mut self,
        centro: Vec2,
        raio_pincel: f32,
        material: MaterialParticula,
        quantidade: usize,
        limites: Rect,
    ) {
        for _ in 0..quantidade {
            if self.particulas.len() >= LIMITE_MAXIMO_PARTICULAS {
                break;
            }
            let angulo = rand::gen_range(0.0, std::f32::consts::TAU);
            let distancia = raio_pincel * rand::gen_range(0.0f32, 1.0f32).sqrt() * 0.82;
            let deslocamento = vec2(angulo.cos(), angulo.sin()) * distancia;
            let margem = 6.0;
            let posicao = vec2(
                (centro.x + deslocamento.x).clamp(limites.x + margem, limites.x + limites.w - margem),
                (centro.y + deslocamento.y).clamp(limites.y + margem, limites.y + limites.h - margem),
            );

            let velocidade = match material {
                MaterialParticula::Elastico => vec2(rand::gen_range(-90.0, 90.0), rand::gen_range(-60.0, 30.0)),
                MaterialParticula::Agua => vec2(rand::gen_range(-35.0, 35.0), rand::gen_range(0.0, 65.0)),
                MaterialParticula::Areia => vec2(rand::gen_range(-18.0, 18.0), rand::gen_range(0.0, 40.0)),
                MaterialParticula::Gas => vec2(rand::gen_range(-45.0, 45.0), rand::gen_range(-60.0, -10.0)),
                MaterialParticula::Fogo => vec2(rand::gen_range(-55.0, 55.0), rand::gen_range(-130.0, -35.0)),
                MaterialParticula::Magnetico => vec2(rand::gen_range(-65.0, 65.0), rand::gen_range(-65.0, 65.0)),
            };

            self.adicionar_particula(posicao, velocidade, material);
        }
    }

    pub fn adicionar_obstaculo(&mut self, inicio: Vec2, fim: Vec2, espessura: f32) {
        if (fim - inicio).length_squared() >= 36.0 {
            self.obstaculos.push(ObstaculoSegmento {
                inicio,
                fim,
                espessura: espessura.clamp(3.5, 14.0),
            });
        }
    }

    pub fn alternar_ancora(&mut self, posicao: Vec2, raio_pincel: f32) {
        let tamanho_antes = self.ancoras.len();
        self.ancoras.retain(|ancora| (ancora.posicao - posicao).length() > 28.0);
        if self.ancoras.len() == tamanho_antes {
            let raio_acao = (raio_pincel * 3.4).clamp(95.0, 280.0);
            self.ancoras.push(AncoraForca {
                posicao,
                modo: self.modo_forca,
                raio_acao,
                fase_animacao: 0.0,
            });
            self.ondas.push(OndaVisual {
                centro: posicao,
                raio_atual: 8.0,
                raio_maximo: raio_acao * 0.65,
                vida_restante: 0.32,
                vida_total: 0.32,
                cor: self.modo_forca.cor(),
            });
        }
    }

    pub fn remover_emissor_proximo(&mut self, posicao: Vec2) -> bool {
        let tamanho_antes = self.emissores.len();
        self.emissores.retain(|emissor| (emissor.posicao - posicao).length() > 26.0);
        self.emissores.len() < tamanho_antes
    }

    pub fn adicionar_emissor(&mut self, posicao: Vec2, alvo_arrasto: Option<Vec2>, material: MaterialParticula) {
        let intervalo = match material {
            MaterialParticula::Fogo | MaterialParticula::Gas => 0.022,
            MaterialParticula::Agua => 0.018,
            MaterialParticula::Areia => 0.024,
            _ => 0.040,
        };

        let (direcao, velocidade_inicial) = if let Some(alvo) = alvo_arrasto
            && (alvo - posicao).length() >= 14.0
        {
            let vetor = alvo - posicao;
            let comprimento = vetor.length();
            (vetor / comprimento, (comprimento * 2.6).clamp(90.0, 440.0))
        } else {
            match material {
                MaterialParticula::Fogo | MaterialParticula::Gas => (vec2(0.0, -1.0), 135.0),
                MaterialParticula::Agua => (vec2(0.45, 0.89).normalize(), 150.0),
                MaterialParticula::Areia => (vec2(0.0, 1.0), 55.0),
                _ => (vec2(0.0, -1.0), 175.0),
            }
        };

        self.emissores.push(Emissor {
            posicao,
            direcao,
            material,
            intervalo,
            acumulador: 0.0,
            velocidade_inicial,
        });
    }

    pub fn apagar_em(&mut self, centro: Vec2, raio_borracha: f32) {
        let raio_quadrado = raio_borracha * raio_borracha;
        self.particulas
            .retain(|particula| (particula.posicao - centro).length_squared() > raio_quadrado);
        self.obstaculos.retain(|obstaculo| {
            let proximo = obstaculo.ponto_mais_proximo(centro);
            (proximo - centro).length() > raio_borracha + obstaculo.espessura
        });
        self.ancoras
            .retain(|ancora| (ancora.posicao - centro).length() > raio_borracha + 10.0);
        self.emissores
            .retain(|emissor| (emissor.posicao - centro).length() > raio_borracha + 10.0);
    }

    pub fn aplicar_forca_cursor(
        &mut self,
        cursor: Vec2,
        velocidade_cursor: Vec2,
        raio_pincel: f32,
        delta_tempo: f32,
    ) {
        let raio_acao = (raio_pincel * 3.4).max(95.0);
        aplicar_campo_em_particulas(
            &mut self.particulas,
            cursor,
            velocidade_cursor,
            raio_acao,
            self.modo_forca,
            delta_tempo,
            1.25,
        );
    }

    pub fn disparar_onda_choque(&mut self, centro: Vec2, raio_pincel: f32) {
        let alcance = (raio_pincel * 4.8).max(160.0);
        for particula in &mut self.particulas {
            let deslocamento = particula.posicao - centro;
            let distancia = deslocamento.length();
            if distancia < alcance {
                let direcao = if distancia > 0.001 {
                    deslocamento / distancia
                } else {
                    vec2(0.0, -1.0)
                };
                let intensidade = (1.0 - distancia / alcance).powi(2) * 560.0;
                particula.velocidade = (particula.velocidade + direcao * intensidade)
                    .clamp_length_max(VELOCIDADE_MAXIMA);
            }
        }
        self.ondas.push(OndaVisual {
            centro,
            raio_atual: 10.0,
            raio_maximo: alcance,
            vida_restante: 0.36,
            vida_total: 0.36,
            cor: Color::new(1.0, 0.82, 0.38, 0.95),
        });
    }

    pub fn atualizar_efeitos_visuais(&mut self, delta_tempo: f32) {
        for ancora in &mut self.ancoras {
            ancora.fase_animacao = (ancora.fase_animacao + delta_tempo * 2.6) % std::f32::consts::TAU;
        }
        for onda in &mut self.ondas {
            onda.vida_restante -= delta_tempo;
            let progresso = 1.0 - (onda.vida_restante / onda.vida_total.max(0.001)).clamp(0.0, 1.0);
            let suavizado = 1.0 - (1.0 - progresso).powi(2);
            onda.raio_atual = 10.0 + (onda.raio_maximo - 10.0) * suavizado;
        }
        self.ondas.retain(|onda| onda.vida_restante > 0.0);
    }

    pub fn atualizar_passo(&mut self, delta_tempo: f32, limites: Rect) {
        self.tempo_acumulado += delta_tempo;
        self.atualizar_emissores(delta_tempo, limites);

        for particula in &mut self.particulas {
            particula.posicao_anterior = particula.posicao;
            if particula.perecivel {
                particula.vida_restante -= delta_tempo;
                let proporcao = (particula.vida_restante / particula.vida_total.max(0.01)).clamp(0.20, 1.0);
                particula.raio = particula.raio_inicial * proporcao.sqrt();
            }
            if particula.condensavel {
                particula.tempo_condensacao -= delta_tempo;
                if particula.tempo_condensacao <= 0.0 {
                    particula.condensar_para_agua();
                }
            }
        }
        self.particulas
            .retain(|particula| !particula.perecivel || particula.vida_restante > 0.0);

        let passo_subdividido = delta_tempo / (SUBPASSOS_FISICA as f32);
        for _ in 0..SUBPASSOS_FISICA {
            self.integrar_forcas_externas(passo_subdividido, limites);
            self.resolver_interacoes_particulas(passo_subdividido, limites);
            self.resolver_colisoes_ambiente(limites);
        }
    }

    fn atualizar_emissores(&mut self, delta_tempo: f32, limites: Rect) {
        let mut novas_emissoes: Vec<(Vec2, Vec2, MaterialParticula)> = Vec::new();
        for emissor in &mut self.emissores {
            emissor.acumulador += delta_tempo;
            let mut geradas_neste_quadro = 0usize;
            while emissor.acumulador >= emissor.intervalo && geradas_neste_quadro < 4 {
                emissor.acumulador -= emissor.intervalo;
                geradas_neste_quadro += 1;
                let desvio_angulo = rand::gen_range(-0.22f32, 0.22f32);
                let seno = desvio_angulo.sin();
                let cosseno = desvio_angulo.cos();
                let direcao_girada = vec2(
                    emissor.direcao.x * cosseno - emissor.direcao.y * seno,
                    emissor.direcao.x * seno + emissor.direcao.y * cosseno,
                );
                let posicao_saida = vec2(
                    (emissor.posicao.x + rand::gen_range(-3.0, 3.0))
                        .clamp(limites.x + 8.0, limites.x + limites.w - 8.0),
                    (emissor.posicao.y + rand::gen_range(-3.0, 3.0))
                        .clamp(limites.y + 8.0, limites.y + limites.h - 8.0),
                );
                let rapidez = emissor.velocidade_inicial * rand::gen_range(0.85, 1.15);
                novas_emissoes.push((posicao_saida, direcao_girada * rapidez, emissor.material));
            }
        }

        for (posicao, velocidade, material) in novas_emissoes {
            self.adicionar_particula(posicao, velocidade, material);
        }
    }

    fn integrar_forcas_externas(&mut self, passo_tempo: f32, limites: Rect) {
        let centro_tela = vec2(limites.x + limites.w * 0.5, limites.y + limites.h * 0.5);

        for ancora in &self.ancoras {
            aplicar_campo_em_particulas(
                &mut self.particulas,
                ancora.posicao,
                Vec2::ZERO,
                ancora.raio_acao,
                ancora.modo,
                passo_tempo,
                0.85,
            );
        }

        for particula in &mut self.particulas {
            let propriedades = particula.material.propriedades();
            let aceleracao_gravidade = match self.modo_gravidade {
                ModoGravidade::Baixo => vec2(0.0, GRAVIDADE_PADRAO * propriedades.escala_gravidade),
                ModoGravidade::Zero => Vec2::ZERO,
                ModoGravidade::Invertida => vec2(0.0, -GRAVIDADE_PADRAO * propriedades.escala_gravidade),
                ModoGravidade::Centro => {
                    let ate_centro = centro_tela - particula.posicao;
                    let distancia = ate_centro.length().max(24.0);
                    let direcao = ate_centro / distancia;
                    direcao * (GRAVIDADE_PADRAO * 0.88 * propriedades.escala_gravidade)
                }
            };

            particula.velocidade += aceleracao_gravidade * passo_tempo;

            match particula.material {
                MaterialParticula::Fogo => {
                    let oscilacao = (self.tempo_acumulado * 9.0 + particula.semente_fase).sin() * 140.0;
                    particula.velocidade.x += (oscilacao + rand::gen_range(-90.0, 90.0)) * passo_tempo;
                }
                MaterialParticula::Gas => {
                    let brisa = (self.tempo_acumulado * 3.5 + particula.semente_fase).cos() * 48.0;
                    particula.velocidade.x += (brisa + rand::gen_range(-45.0, 45.0)) * passo_tempo;
                    particula.velocidade.y += rand::gen_range(-35.0, 35.0) * passo_tempo;
                }
                _ => {}
            }

            let amortecimento = propriedades.amortecimento_ar.powf(passo_tempo * 60.0);
            particula.velocidade *= amortecimento;
            particula.velocidade = particula.velocidade.clamp_length_max(VELOCIDADE_MAXIMA);
            particula.posicao += particula.velocidade * passo_tempo;
        }
    }

    fn resolver_interacoes_particulas(&mut self, passo_tempo: f32, limites: Rect) {
        self.grade.reconstruir(limites, &self.particulas);
        let total = self.particulas.len();
        let total_colunas = self.grade.colunas() as isize;
        let total_linhas = self.grade.linhas() as isize;

        for indice_a in 0..total {
            let (coluna_base, linha_base) = self.grade.coordenadas_celula(self.particulas[indice_a].posicao);
            let alcance = if self.particulas[indice_a].material == MaterialParticula::Magnetico {
                2isize
            } else {
                1isize
            };

            let linha_inicio = (linha_base as isize - alcance).max(0);
            let linha_fim = (linha_base as isize + alcance).min(total_linhas - 1);
            let coluna_inicio = (coluna_base as isize - alcance).max(0);
            let coluna_fim = (coluna_base as isize + alcance).min(total_colunas - 1);

            for linha in linha_inicio..=linha_fim {
                for coluna in coluna_inicio..=coluna_fim {
                    let mut cursor_lista = self.grade.cabeca_em(coluna as usize, linha as usize);
                    while cursor_lista >= 0 {
                        let indice_b = cursor_lista as usize;
                        cursor_lista = self.grade.proximo_de(indice_b);

                        if indice_b <= indice_a {
                            continue;
                        }

                        let (esquerda, direita) = self.particulas.split_at_mut(indice_b);
                        let particula_a = &mut esquerda[indice_a];
                        let particula_b = &mut direita[0];
                        resolver_par_particulas(particula_a, particula_b, passo_tempo, indice_a, indice_b);
                    }
                }
            }
        }
    }

    fn resolver_colisoes_ambiente(&mut self, limites: Rect) {
        for particula in &mut self.particulas {
            let propriedades = particula.material.propriedades();

            for obstaculo in &self.obstaculos {
                let ponto_proximo = obstaculo.ponto_mais_proximo(particula.posicao);
                let diferenca = particula.posicao - ponto_proximo;
                let raio_combinado = particula.raio + obstaculo.espessura;
                let distancia_quadrada = diferenca.length_squared();

                if distancia_quadrada < raio_combinado * raio_combinado {
                    let distancia = distancia_quadrada.sqrt();
                    let normal = if distancia > 0.0001 {
                        diferenca / distancia
                    } else {
                        vec2(0.0, -1.0)
                    };
                    let penetracao = raio_combinado - distancia;
                    particula.posicao += normal * penetracao;

                    let velocidade_normal = particula.velocidade.dot(normal);
                    if velocidade_normal < 0.0 {
                        let restituicao = if -velocidade_normal < 16.0 {
                            0.0
                        } else {
                            propriedades.restituicao
                        };
                        let componente_tangencial = particula.velocidade - normal * velocidade_normal;
                        let fator_atrito = (1.0 - propriedades.atrito * 0.55).clamp(0.15, 1.0);
                        particula.velocidade =
                            componente_tangencial * fator_atrito - normal * (velocidade_normal * restituicao);
                    }
                }
            }

            let fator_tangencial = (1.0 - propriedades.atrito * 0.35).clamp(0.35, 0.998);
            if particula.posicao.x - particula.raio < limites.x {
                particula.posicao.x = limites.x + particula.raio;
                if particula.velocidade.x < 0.0 {
                    let restituicao = if -particula.velocidade.x < 16.0 { 0.0 } else { propriedades.restituicao };
                    particula.velocidade.x = -particula.velocidade.x * restituicao;
                    particula.velocidade.y *= fator_tangencial;
                }
            }
            if particula.posicao.x + particula.raio > limites.x + limites.w {
                particula.posicao.x = limites.x + limites.w - particula.raio;
                if particula.velocidade.x > 0.0 {
                    let restituicao = if particula.velocidade.x < 16.0 { 0.0 } else { propriedades.restituicao };
                    particula.velocidade.x = -particula.velocidade.x * restituicao;
                    particula.velocidade.y *= fator_tangencial;
                }
            }
            if particula.posicao.y - particula.raio < limites.y {
                particula.posicao.y = limites.y + particula.raio;
                if particula.velocidade.y < 0.0 {
                    let restituicao = if -particula.velocidade.y < 16.0 { 0.0 } else { propriedades.restituicao };
                    particula.velocidade.y = -particula.velocidade.y * restituicao;
                    particula.velocidade.x *= fator_tangencial;
                }
            }
            if particula.posicao.y + particula.raio > limites.y + limites.h {
                particula.posicao.y = limites.y + limites.h - particula.raio;
                if particula.velocidade.y > 0.0 {
                    let restituicao = if particula.velocidade.y < 16.0 { 0.0 } else { propriedades.restituicao };
                    particula.velocidade.y = -particula.velocidade.y * restituicao;
                    particula.velocidade.x *= fator_tangencial;
                }
            }

            if !particula.posicao.is_finite() || !particula.velocidade.is_finite() {
                particula.posicao = vec2(limites.x + limites.w * 0.5, limites.y + limites.h * 0.5);
                particula.velocidade = Vec2::ZERO;
            }
        }
    }
}

fn aplicar_campo_em_particulas(
    particulas: &mut [Particula],
    origem: Vec2,
    velocidade_referencia: Vec2,
    raio_acao: f32,
    modo: ModoForca,
    passo_tempo: f32,
    ganho: f32,
) {
    let raio_quadrado = raio_acao * raio_acao;
    for particula in particulas {
        let deslocamento = particula.posicao - origem;
        let distancia_quadrada = deslocamento.length_squared();
        if distancia_quadrada >= raio_quadrado {
            continue;
        }
        let distancia = distancia_quadrada.sqrt();
        let direcao = if distancia > 0.001 {
            deslocamento / distancia
        } else {
            vec2(0.0, -1.0)
        };
        let decaimento = 1.0 - (distancia / raio_acao);

        match modo {
            ModoForca::Repelir => {
                let intensidade = decaimento * decaimento * 1450.0 * ganho;
                particula.velocidade += direcao * intensidade * passo_tempo;
            }
            ModoForca::Atrair => {
                let suavizacao_centro = (distancia / 22.0).clamp(0.18, 1.0);
                let intensidade = decaimento * 1350.0 * suavizacao_centro * ganho;
                particula.velocidade -= direcao * intensidade * passo_tempo;
                if distancia < 26.0 {
                    particula.velocidade *= (1.0 - 2.2 * passo_tempo).clamp(0.85, 1.0);
                }
            }
            ModoForca::Vortice => {
                let tangente = vec2(-direcao.y, direcao.x);
                let giro = decaimento * 1380.0 * ganho;
                let puxao = decaimento * 420.0 * ganho;
                particula.velocidade += (tangente * giro - direcao * puxao) * passo_tempo;
            }
            ModoForca::Arrastar => {
                let alvo = velocidade_referencia * 0.92;
                let acoplamento = (decaimento * 12.0 * passo_tempo * ganho).clamp(0.0, 0.65);
                particula.velocidade += (alvo - particula.velocidade) * acoplamento;
                let centralizacao = decaimento * 180.0 * ganho;
                particula.velocidade -= direcao * centralizacao * passo_tempo;
            }
        }

        particula.velocidade = particula.velocidade.clamp_length_max(VELOCIDADE_MAXIMA);
    }
}

fn resolver_par_particulas(
    particula_a: &mut Particula,
    particula_b: &mut Particula,
    passo_tempo: f32,
    indice_a: usize,
    indice_b: usize,
) {
    let diferenca = particula_b.posicao - particula_a.posicao;
    let distancia_quadrada = diferenca.length_squared();
    let soma_raios = particula_a.raio + particula_b.raio;

    let alcance_maximo = if particula_a.material == MaterialParticula::Magnetico
        && particula_b.material == MaterialParticula::Magnetico
    {
        RAIO_INTERACAO_MAGNETICA
    } else if particula_a.material == MaterialParticula::Agua
        && particula_b.material == MaterialParticula::Agua
    {
        RAIO_INTERACAO_FLUIDO.max(soma_raios)
    } else {
        (soma_raios * 1.45).max(soma_raios)
    };

    if distancia_quadrada > alcance_maximo * alcance_maximo {
        return;
    }

    let distancia = distancia_quadrada.sqrt();
    let normal = if distancia > 0.0001 {
        diferenca / distancia
    } else {
        let angulo_desempate = ((indice_a * 73 + indice_b * 37) % 360) as f32 * 0.017453292;
        vec2(angulo_desempate.cos(), angulo_desempate.sin())
    };

    if particula_a.material == MaterialParticula::Fogo || particula_b.material == MaterialParticula::Fogo {
        let limite_reacao = soma_raios * 1.38;
        if distancia < limite_reacao {
            if particula_a.material == MaterialParticula::Fogo && particula_b.material == MaterialParticula::Agua {
                particula_a.vida_restante -= 3.2 * passo_tempo;
                particula_b.converter_para_vapor();
            } else if particula_b.material == MaterialParticula::Fogo
                && particula_a.material == MaterialParticula::Agua
            {
                particula_b.vida_restante -= 3.2 * passo_tempo;
                particula_a.converter_para_vapor();
            } else if particula_a.material == MaterialParticula::Fogo
                && particula_b.material == MaterialParticula::Gas
                && particula_b.inflamavel
            {
                particula_b.inflamar();
            } else if particula_b.material == MaterialParticula::Fogo
                && particula_a.material == MaterialParticula::Gas
                && particula_a.inflamavel
            {
                particula_a.inflamar();
            }
        }
    }

    if particula_a.material == MaterialParticula::Agua
        && particula_b.material == MaterialParticula::Agua
        && distancia < RAIO_INTERACAO_FLUIDO
        && distancia > soma_raios * 0.92
    {
        let proporcao = 1.0 - distancia / RAIO_INTERACAO_FLUIDO;
        let coesao = proporcao * 145.0 * passo_tempo;
        particula_a.velocidade += normal * coesao;
        particula_b.velocidade -= normal * coesao;

        let diferenca_velocidade = particula_b.velocidade - particula_a.velocidade;
        let viscosidade = (proporcao * 0.14).clamp(0.0, 0.18);
        particula_a.velocidade += diferenca_velocidade * viscosidade;
        particula_b.velocidade -= diferenca_velocidade * viscosidade;
    }

    if particula_a.material == MaterialParticula::Magnetico
        && particula_b.material == MaterialParticula::Magnetico
        && distancia < RAIO_INTERACAO_MAGNETICA
        && distancia > soma_raios * 0.75
    {
        let produto_cargas = particula_a.sinal_carga * particula_b.sinal_carga;
        let envolvente = 1.0 - (distancia / RAIO_INTERACAO_MAGNETICA);
        let intensidade = -produto_cargas * 9800.0 * envolvente / (distancia + 10.0);
        let impulso_magnetico = normal * (intensidade * passo_tempo);
        particula_a.velocidade -= impulso_magnetico * particula_a.massa_inversa * 18.0;
        particula_b.velocidade += impulso_magnetico * particula_b.massa_inversa * 18.0;
    }

    if distancia < soma_raios {
        let soma_massa_inversa = particula_a.massa_inversa + particula_b.massa_inversa;
        if soma_massa_inversa <= 0.0 {
            return;
        }

        let penetracao = soma_raios - distancia;
        let fator_relaxamento = if particula_a.material == MaterialParticula::Gas
            || particula_b.material == MaterialParticula::Gas
            || particula_a.material == MaterialParticula::Fogo
            || particula_b.material == MaterialParticula::Fogo
        {
            0.35
        } else {
            0.78
        };
        let correcao = normal * (penetracao * fator_relaxamento / soma_massa_inversa);
        particula_a.posicao -= correcao * particula_a.massa_inversa;
        particula_b.posicao += correcao * particula_b.massa_inversa;

        let velocidade_relativa = particula_b.velocidade - particula_a.velocidade;
        let velocidade_normal = velocidade_relativa.dot(normal);

        if velocidade_normal < 0.0 {
            let propriedades_a = particula_a.material.propriedades();
            let propriedades_b = particula_b.material.propriedades();
            let restituicao_base = propriedades_a.restituicao.min(propriedades_b.restituicao);
            let restituicao_efetiva = if -velocidade_normal < 18.0 {
                0.0
            } else {
                restituicao_base
            };

            let magnitude_impulso = -(1.0 + restituicao_efetiva) * velocidade_normal / soma_massa_inversa;
            let impulso_normal = normal * magnitude_impulso;
            particula_a.velocidade -= impulso_normal * particula_a.massa_inversa;
            particula_b.velocidade += impulso_normal * particula_b.massa_inversa;

            let vetor_tangencial = velocidade_relativa - normal * velocidade_normal;
            let comprimento_tangencial = vetor_tangencial.length();
            if comprimento_tangencial > 0.001 {
                let tangente = vetor_tangencial / comprimento_tangencial;
                let atrito_medio = (propriedades_a.atrito + propriedades_b.atrito) * 0.5;
                let limite_atrito = magnitude_impulso * atrito_medio;
                let magnitude_tangencial =
                    (-comprimento_tangencial / soma_massa_inversa).clamp(-limite_atrito, limite_atrito);
                let impulso_tangencial = tangente * magnitude_tangencial;
                particula_a.velocidade -= impulso_tangencial * particula_a.massa_inversa;
                particula_b.velocidade += impulso_tangencial * particula_b.massa_inversa;
            }
        }

        if particula_a.material == MaterialParticula::Areia
            && particula_b.material == MaterialParticula::Areia
            && velocidade_relativa.length_squared() < 225.0
        {
            particula_a.velocidade *= 0.90;
            particula_b.velocidade *= 0.90;
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::cenas::Cena;

    #[test]
    fn colisao_elastica_separa_particulas_e_conserva_momento() {
        let mut mundo = MundoSimulacao::novo();
        mundo.modo_gravidade = ModoGravidade::Zero;
        let limites = Rect::new(0.0, 0.0, 800.0, 600.0);

        let mut primeira = Particula::nova(vec2(396.0, 300.0), vec2(180.0, 0.0), MaterialParticula::Elastico, true);
        let mut segunda = Particula::nova(vec2(404.0, 300.0), vec2(-180.0, 0.0), MaterialParticula::Elastico, false);
        primeira.raio = 6.0;
        primeira.massa_inversa = 1.0 / 36.0;
        segunda.raio = 6.0;
        segunda.massa_inversa = 1.0 / 36.0;

        mundo.particulas.push(primeira);
        mundo.particulas.push(segunda);

        mundo.atualizar_passo(1.0 / 60.0, limites);

        let distancia = (mundo.particulas[1].posicao - mundo.particulas[0].posicao).length();
        assert!(distancia >= 11.5);
        assert!(mundo.particulas[0].velocidade.x < 0.0);
        assert!(mundo.particulas[1].velocidade.x > 0.0);
        let soma_velocidades_x = mundo.particulas[0].velocidade.x + mundo.particulas[1].velocidade.x;
        assert!(soma_velocidades_x.abs() < 1.0);
    }

    #[test]
    fn reacao_fogo_e_agua_gera_vapor_condensavel() {
        let mut mundo = MundoSimulacao::novo();
        mundo.modo_gravidade = ModoGravidade::Zero;
        let limites = Rect::new(0.0, 0.0, 600.0, 600.0);

        mundo.adicionar_particula(vec2(300.0, 300.0), Vec2::ZERO, MaterialParticula::Fogo);
        mundo.adicionar_particula(vec2(303.0, 300.0), Vec2::ZERO, MaterialParticula::Agua);

        mundo.atualizar_passo(1.0 / 60.0, limites);

        let encontrou_vapor = mundo
            .particulas
            .iter()
            .any(|particula| particula.material == MaterialParticula::Gas && particula.condensavel);
        assert!(encontrou_vapor);
    }

    #[test]
    fn reacao_fogo_e_gas_inflamavel_propaga_combustao() {
        let mut mundo = MundoSimulacao::novo();
        mundo.modo_gravidade = ModoGravidade::Zero;
        let limites = Rect::new(0.0, 0.0, 600.0, 600.0);

        mundo.adicionar_particula(vec2(300.0, 300.0), Vec2::ZERO, MaterialParticula::Fogo);
        mundo.adicionar_particula(vec2(304.0, 300.0), Vec2::ZERO, MaterialParticula::Gas);

        mundo.atualizar_passo(1.0 / 60.0, limites);

        let todas_em_fogo = mundo
            .particulas
            .iter()
            .all(|particula| particula.material == MaterialParticula::Fogo);
        assert!(todas_em_fogo);
    }

    #[test]
    fn obstaculo_segmento_bloqueia_queda_de_particula() {
        let mut mundo = MundoSimulacao::novo();
        let limites = Rect::new(0.0, 0.0, 600.0, 600.0);
        mundo.adicionar_obstaculo(vec2(200.0, 320.0), vec2(400.0, 320.0), 6.0);
        mundo.adicionar_particula(vec2(300.0, 305.0), vec2(0.0, 120.0), MaterialParticula::Areia);

        for _ in 0..30 {
            mundo.atualizar_passo(1.0 / 60.0, limites);
        }

        assert!(mundo.particulas[0].posicao.y < 320.0);
    }

    #[test]
    fn todas_as_cenas_executam_estavelmente_dentro_dos_limites() {
        let mut mundo = MundoSimulacao::novo();
        let limites = Rect::new(0.0, 42.0, 1280.0, 738.0);
        let cenas = [
            Cena::Livre,
            Cena::Ampulheta,
            Cena::Cascata,
            Cena::Orbita,
            Cena::Alquimia,
        ];

        for cena in cenas {
            cena.aplicar(&mut mundo, limites);
            for _ in 0..45 {
                mundo.atualizar_passo(1.0 / 60.0, limites);
                mundo.atualizar_efeitos_visuais(1.0 / 60.0);
            }
            assert!(!mundo.particulas.is_empty());
            for particula in &mundo.particulas {
                assert!(particula.posicao.is_finite());
                assert!(particula.velocidade.is_finite());
                assert!(particula.posicao.x >= limites.x);
                assert!(particula.posicao.x <= limites.x + limites.w);
                assert!(particula.posicao.y >= limites.y);
                assert!(particula.posicao.y <= limites.y + limites.h);
            }
        }
    }
}
