# A01 v1.3 — comandos administrativos compatíveis

EntradaA00, projeto novo, broker/corev0.11.5 imutável. Antes de editar: copiar
por patch os blobsGit src/bin/mqtt-admin/{main,users,state}.rs, preservar
copyright/MIT e registrar hashes de origem. Adaptar nome/versão para xmqr-admin;
manter user create/list/update-password/delete, hash-password, fingerprint,
state backup/verify/restore e comportamento offline/lock/symlink/arquivo privado.
Adicionar help seguro. Não copiar/reimplementar WAL ou codec MQTT.

Arquivos: Cargo.toml/lock, src/{main,cli,users,state}.rs, tests/cli.rs,
LICENSE, ORIGIN.json, scripts/license_inventory.py, docs/third-party-licenses.md,
README, AGENTS, VERSIONING, CHANGELOG, SECURITY, CI e registroA01.
Gates:fmt/test/clippy-Dwarnings/buildlockedrelease e inventário. Negativos:
comando inválido sem echo; lock ativo, backup íntegro/adulterado, restore novo,
preservação de fingerprints, symlink e permissõesUnix. Comparar state com
broker/core pinado e validar install side-by-side com mqtt-admin legado.

Compatibilidade: nome novo impede conflito de pacotes; arquivo legado e backups
mesmos formatos. CRUD mantém escopo laboratórios; securebundle ainda manual.
WSL/Linux é alvo de persistência. Windows nativo não homologado. Sem secretos
como argumento CLI; senhas interativas. Não promover feature não existente.

Revisão após gate: help retornou0 corretamente, mas sua descrição não continha
o recorte explícito “somente leitura” exigido pelo teste novo. Clarear o help
antes de reexecutar, sem alterar outras operações ou inventar suporte remoto.

Clippy1.99 exigiu assert_eq para saídas vazias no testeCLI; ajustar as duas
asserções, preservando a exigência de stdout/stderr vazios, sem relaxar lint.

Gate ampliado de senha emPTY falhou. Primeiro emitir apenas diagnóstico
sanitizado (exit/prompts/echo booleanos), sem stdout/senha/hash. Se for corrida
de escrita antes deTTY desativarECHO, aguardar estado termios apropriado no
harness; não remover a asserção de não vazamento nem alterar CLI sem reprodução.
