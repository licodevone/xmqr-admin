# A18 rev1 — admin compatível com estado v14 e marco 0.9.0

Entrada: admin independente 0.8.0 fixa broker 0.26.0/estado v13. P156/P157 introduzem proveniência de cópia compartilhada QoS1 no documento v14. Este prompt antecede qualquer alteração A18.

Objetivo: após CI do broker 0.27.0, fixar seu commit Git completo em Cargo.toml/Cargo.lock, atualizar inventário de licenças e validar inspeção e backup/verify/restore offline v14 com origem do grupo, bit Retain e obrigação QoS1 preservados. Manter perfil de listener explícito, CLI/canal local e painel somente leitura. Após gates, elevar versão própria a 0.9.0 e publicar pacote/imagem numerados, tag anotada e prerelease com hashes.

Aceite: Rust, fmt, Clippy, build release, licenças, contratos pacote/export e CI; laboratório descartável criando v14 pelo broker P156, backup/restore sem perder obrigação, lock ativo e snapshot adulterado recusados; integração de canal com broker 0.27.0. Ciclo `.deb` Ubuntu 26.04 amd64, Docker por digest, anexos remotos SHA-256/tamanho. Preservar backup privado v13 para rollback, não mover `latest`, não instalar no WSL do operador nem alegar Windows/arm64/produção/fase 9 completa.
