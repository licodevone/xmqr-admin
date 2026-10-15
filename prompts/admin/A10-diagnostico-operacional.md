# A10 rev1 — diagnóstico operacional de recuperação

2026-10-15. Entrada admin `a007307` mais A09 local; broker P112 em preparação. A08 requer dry-run, espaço livre, backup íntegro, versões e saída redigida.

Implementar `diagnose` somente leitura no Linux/WSL. Receber diretório de estado, bundle privado, backup existente, socket e token viewer. Validar bundle e backup pelas APIs existentes, consultar status/versão do broker, medir espaço livre no filesystem do estado com API segura e emitir JSON limitado com resultados/versões/bytes, sem caminhos, hashes, IDs, tokens, credenciais ou payload. Falha em qualquer verificação torna exit code não zero; não abrir WAL ativo.

Negativos para bundle/backup inválido, socket indisponível e falta de espaço; integração real após gates fmt/test/Clippy/build/licenças. Registrar evidência antes de aceitar A08.
