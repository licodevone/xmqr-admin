# A14 rev1 — admin compatível com estado v11 e marco 0.6.0

Entrada: admin 0.5.0 contém core broker 0.23.0 e lê estado até v10; P148 introduz No Local/Retain As Published e documento v11, com broker 0.24.0 planejado em P149. Este prompt precede qualquer alteração A14. Esperar tag publicada `v0.24.0` do broker e fixar o commit exato da tag, nunca um ramo móvel ou dependência de caminho.

Objetivo: atualizar o pin Git imutável e Cargo.lock ao core broker 0.24.0, validar bundle e backup/verify/restore offline de estado v11 com opções de assinatura persistidas. Manter o perfil `.mqtt-broker-profile` fora do backup e exigir seleção explícita ao reabrir estado restaurado. Preservar CLI/canal local e painel somente leitura. Após CI e distribuição testados, elevar versão própria para 0.6.0 e publicar pacote/imagem numerados, tag anotada e prerelease com hashes.

Aceite: testes Rust, fmt, Clippy, build release, licenças, contratos de empacotamento/exportação e CI; laboratório descartável com broker 0.24 que cria sessão v11 e comprova backup/restore sem perda das opções, recusa de lock ativo e snapshot adulterado. Ciclo de `.deb` Ubuntu26 amd64, probe Docker com broker/client da mesma família, digest remoto e anexos GitHub verificados. Não copiar marcadores automaticamente, instalar no WSL do operador, mover `latest`, alegar Windows/arm64/produção nem declarar fase 9 fechada.
