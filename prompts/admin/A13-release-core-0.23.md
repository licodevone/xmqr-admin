# A13 rev1 — marco experimental do admin com core broker 0.23.0

Entrada: A12 alinhou o pin Git imutável ao broker 0.23.0, exercitou estado v10 em laboratório e precisa de CI PASS antes do aceite. A versão distribuída do admin ainda é 0.4.0 e contém o core anterior. Este prompt precede a mudança de versão.

Objetivo: após aceite de A12, publicar o admin independente 0.5.0 com o core 0.23.0. Atualizar Cargo, lock, Dockerfile, changelog e documentação. Manter painel somente leitura e canal local. Não tratar o marcador `.mqtt-broker-profile` como parte do backup; documentar a escolha explícita do perfil ao restaurar.

Aceite: testes Rust, fmt, Clippy, build release, inventário de licenças, contratos de distribuição/exportação, integração v10 offline e CI do commit exato. Empacotar tar/.deb Ubuntu26 amd64 e imagem `lepinheiro100/xmqr-admin-ui:0.5.0` com revisão/versão corretas; ensaiar ciclo de pacote em contêiner descartável e runtime Docker isolado. Só então criar tag anotada `v0.5.0`, prerelease GitHub, anexos com SHA256, push da imagem numerada e verificar digest remoto. Registrar hashes, limites e rollback. Não mover `latest`, tags antigas, instalar no WSL do operador nem alegar suporte Windows/arm64/produção.
