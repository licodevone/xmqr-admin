# A20 rev1 — marco experimental admin 0.10.0, core v15

Entrada: A19 fixa broker P164/v15 e comprova backup/verify/restore de Will pendente. O admin 0.9.0 copia backup bruto v15, mas broker 0.28.0 rejeita o estado; este prompt precede qualquer mudança de versão.

Objetivo: distribuir admin independente 0.10.0 com core imutável do broker v15, preservando painel somente leitura e perfil de listener externo. Atualizar Cargo/lock, Dockerfile/empacotamento, CHANGELOG, README, VERSIONING e nota de release. Explicitar rollback executável pelo backup privado v14 anterior, sem sugerir que o admin antigo destrói backup bruto v15.

Aceite: A19 real com broker P164 e pacotes antigos, testes Rust/fmt/Clippy/build release/licenças/distribuição/CI exata, pacote Ubuntu 26.04 amd64 e ciclo descartável de upgrade/rollback, imagem numerada executada por digest, tag anotada e prerelease com manifesto/SHA-256/tamanhos remotos conferidos. Sem mover `latest`, instalar no WSL do operador ou declarar fases globais encerradas.
