# A07 rev1 — prerelease do editor offline 0.3.0

2026-10-15. Entrada: A06 aceito em `c5e51ac`, 21 testes unitários, seis
integrações Rust, fmt/Clippy/build/inventário PASS em Ubuntu26. Admin 0.2.0
continua a última release. Broker fase5 A09 ainda aberto.

Publicar versão experimental 0.3.0 do admin independente com editor offline
do bundle seguro. Alinhar Cargo.toml/lock, Dockerfile, README/changelog, repetir
gates após bump, construir .deb/tar Ubuntu26 e imagem Docker numerada do commit
limpo, exportar hashes/manifesto e comparar assets/digest remotos. Tag imutável
e prerelease GitHub; não mover latest nem anunciar fase5 completa. Manter
compatibilidade do painel de leitura e CLI legado; sem operação online, WAL
ou serviço novo. Registrar rollback para 0.2.0 e limites experimentais.
