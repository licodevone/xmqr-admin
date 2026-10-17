# A16 rev1 — admin compatível com estado v13 e marco 0.8.0

Entrada: admin 0.7.0 fixa core broker 0.25.0/estado v12; P154 introduz associação compartilhada durável no documento v13. Este prompt precede alterações A16. Fixar o commit exato da tag broker 0.26.0 quando P154 e a fonte de distribuição tiverem CI aprovado; não usar ramo móvel nem dependência de caminho.

Objetivo: atualizar pin Git imutável, Cargo.lock e inventário de licenças; validar inspeção, backup/verify/restore offline de v13 com `$share/grupo/filtro`, Subscription Identifier, fila e inflight QoS2 da sessão escolhida preservados. Perfil de listener segue separado e explícito. Manter CLI/canal local e painel somente leitura; não editar WAL com broker ativo. Após gates e distribuição completos, elevar versão própria a 0.8.0 e publicar pacote/imagem numerados, tag anotada e prerelease com hashes.

Aceite: Rust, fmt, Clippy, build release, licenças, contratos de pacote/export e CI; laboratório descartável com broker P154 criando v13, backup/restore sem perder associação/QoS2, lock ativo e snapshot adulterado recusados. Ciclo `.deb` Ubuntu26 amd64, Docker numerado executado por digest, anexos GitHub SHA-256/tamanho verificados. Preservar rollback por backup privado v12, não mover `latest`, não instalar no WSL do operador nem declarar Windows/arm64/produção ou fase 9 completa.
