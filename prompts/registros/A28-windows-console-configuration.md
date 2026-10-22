# A28 - Console MQTT configuravel no Windows

Prompt: [A28](../admin/A28-windows-console-configuration.md), revisao1.2.
Base86f0dd917783da26ac1654526dd0c583d161733c, versao0.11.0 preservada.
Arvore limpa na entrada, exceto prompt preparado pelo agente principal.

## Implementacao

Config aceita a secao opcional estrita `[mqtt_console]`, com client_bin e
sub_profile obrigatorios; pub_profile opcional usa sub_profile por padrao;
label opcional usa XMQR local. Caminhos relativos sao resolvidos contra a
pasta do TOML, com entrada de1..4096bytes sem controles; label1..80bytes sem
controles. Limite global64KiB e schema/version existentes preservados.

web::serve entrega a fonte explicita ao Board, sem alterar ambiente global.
Secao ausente chama exatamente o fluxo Board::from_env anterior. Validacao de
perfis privados, binario absoluto regular, geracao aleatoria de client-id,
revalidacao de perfil antes das acoes, autenticacao/RBAC/CSRF/origem, limites
de payload/fila/deadline/lease e controle de filhos permanecem existentes.
Construir Board nao inicia processo nem conexao MQTT.

packaging/windows-service/build.rs compila recurso1ICON com rc.exe SDK e
passa .res somente ao link de xmqr-admin. assets/admin.ico e copia original
do Broker, SHA256
`a48b0338b4fb442378aad141c544228d8b33fa3cdcb4ca392f15f854589aee91`.
Sem dependencia Rust adicional, RUSTFLAGS global ou edicao PE apos o link.
Os manifests e locks dos dois perfis permaneceram inalterados.

## Execucao real

- Primeiro Clippy all-targets: FAIL E0308 no novo teste ACL, por handle
  imutavel. Prompt revisado1.1 antes da correcao para handle mutavel com
  READ_CONTROL/WRITE_DAC, somente no fixture proprio.
- Segundo Clippy: FAIL items_after_statements no import do teste. Prompt
  revisado1.2 antes de mover import; nenhum allow ou validador alterado.
- Clippy final all-targets -Dwarnings: PASS1.91s.
- Suite Windows-service:36/36 PASS9.55s (compilacao28.46s). Inclui cinco
  novos casos para config/defaults/negativos, selecao explicita sem fallback,
  ausencia de filho, perfil ausente/publico/oversize e label invalido, alem
  dos negativos web/CSRF/autorizacao e stores privados preexistentes.
- fmt --all --check: PASS; git diff --check: PASS.
- Release Windows MSVC locked/offline: PASS22.29s.
- Executavel resultante executou capabilities e version: schema1/admin/
  service_entrypointtrue, xmqr-admin0.11.0.
- Inventario de licencas do manifesto CLI normal: BLOCKED offline. Cache
  xp242 nao possui cc1.8.0 requerido pelo lock normal; comando falhou ao
  resolver metadata antes da comparacao. Nenhuma dependencia/lock foi mudada
  para ocultar isso. Nao alegar gatePASS nem build/teste CLI normal/Unix.

Gates usam o manifesto packaging/windows-service e target/windows-msi,
CARGO_HOME curto xp242-cargo e CARGO_INCREMENTAL=0. Testes usam TMP/TEMP
exclusivo no Temp do usuario; todos os segredos/ACLs de fixture sao efemeros.
Nao foram lidos segredos, alterados servicos, instalacao ou ACLs do host.

Artefato Windows nativo final, sem pos-edicao:
`target/windows-msi/x86_64-pc-windows-msvc/release/xmqr-admin.exe`,
4.350.976bytes, SHA256
`42687cfe339a593fa8b0fff479bc833322ce2da1dea0d5160735aaaae0b4a8bc`.
Agente principal confirmou auditoria readonly do icone:9imagens/10recursos,
SHA256 acima preservado. E2E isolado P252 PASS no relatorio
`../xmqr/target/p252-console-proof/report.json`, lido apos execucao do agente
responsavel: AdminA28 e ClientP248 com hashes esperados, Broker mTLS proprio,
config TOML habilitando console, bootstrap/logout/login com cookie+CSRF,
negativos sem autenticacao/CSRF, assinatura QoS1 conectada, publicacao QoS1
e recepcao da mesma mensagem sintetica, disconnect e filhos Admin/Broker
encerrados cooperativamente com exit0. SCM/MSI e configuracao instalada
nao foram executados/lidos nessa prova. Nenhum funcionamento instalado e
inferido desses testes.
Sem commit/push/tag/release ou atualizacao automatica do host.
