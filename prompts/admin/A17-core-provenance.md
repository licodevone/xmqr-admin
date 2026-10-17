# A17 — proveniência do core no pacote administrativo

Entrada: o candidato A16/0.8.0 contém o pin Git correto do broker 0.26.0 no Cargo.toml/Cargo.lock, mas o construtor de distribuição ainda escreve no manifesto e no tar/deb um `core_revision` fixo da família antiga. Este prompt antecede a correção do gerador e invalida os artefatos A16 produzidos antes dela.

Objetivo: derivar a revisão do core a partir do pin Git completo do Cargo.toml e exigir correspondência exata com a origem resolvida no Cargo.lock. Falhar antes de criar o diretório de saída se o pin estiver ausente, abreviado, móvel ou divergente. Usar essa revisão validada no manifesto e na proveniência dentro do tar/deb. Cobrir casos positivos e negativos com teste de contrato.

Aceite: teste de contrato, fmt/Clippy/Rust e CI apropriados, reconstrução r1/r2 no Ubuntu 26.04 descartável, inspeção da proveniência em ambos os formatos, ciclo de pacote novamente e SHA-256 dos anexos após publicar. Não reaproveitar pacotes ou imagem identificados por commit anterior, não criar tag antes dos gates do novo commit.
