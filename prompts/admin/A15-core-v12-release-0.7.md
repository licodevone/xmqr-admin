# A15 rev1 — admin compatível com estado v12 e marco 0.7.0

Entrada: admin 0.6.0 contém core broker 0.24.0 e lê até estado v11; P150 introduz Subscription Identifier e v12, a ser distribuído em broker 0.25.0/P151. Este prompt precede qualquer alteração A15. Esperar tag `v0.25.0` publicada e fixar o commit exato da tag, nunca ramo móvel ou dependência de caminho.

Objetivo: atualizar o pin Git imutável e Cargo.lock ao core broker 0.25.0, validar bundle e backup/verify/restore offline de v12 com IDs de assinatura preservados. Perfil `.mqtt-broker-profile` segue fora do backup e requer seleção explícita. Preservar CLI/canal local e painel somente leitura. Após CI e distribuição testados, elevar versão própria a 0.7.0 e publicar pacote/imagem numerados, tag anotada e prerelease com hashes.

Aceite: testes Rust, fmt, Clippy, build release, licenças, contratos de pacote/export e CI; laboratório descartável com broker 0.25 criando sessão v12, backup/restore sem perda dos IDs, recusa de lock ativo/snapshot adulterado. Ciclo `.deb` Ubuntu26 amd64, probe Docker com broker/client da mesma família, digest remoto e anexos GitHub verificados. Não copiar marcador automaticamente, instalar no WSL do operador, mover `latest`, alegar Windows/arm64/produção ou fechar fase 9.
