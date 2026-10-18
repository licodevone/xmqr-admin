# A21 — fixture preparada, oráculo v15 corrigido, integração16 não executada

18/10/2026. Entrada e5c4f23eef70c4b22427e5c763532e2e1a670dec,
admin0.10.0/coreGit9215d4c (v15). PromptA21 v1.0 precedeu o código;
revisão v1.1 detalhou isolamento, oráculo e contratos antes do script novo.
Ownership desta contribuição: prompt, scripts/verify_core_v16_expiry.py e
este registro. Cargo/lock/pin/versão/CI/distribuição pertencem ao coordenador.

## Resultado real até este checkpoint

| Gate | Estado |
| --- | --- |
| Oito contratos Python no próprio módulo | PASS,0.072s |
| git diff --check | PASS |
| Fixture real broker16/admin pinado/legacy15 | NOT_RUN: aguardando os ELFs exatos do coordenador |
| Cargo/build/test/Clippy/admin pin/release | Não executados por este agente nesta contribuição |

Os contratos verificam parsing estruturado MQSNAP01/CRC32/TOML, tamanho e
truncamento, chave duplicada/trailing, string version16 no payload que não é
envelope, PUBLISH/propriedades e varints, manifesto/hashes/path/entrada extra,
intervalo nativo versus legado e limpeza das variáveis MQTT herdadas. O erro
argparse sem os quatro argumentos é esperado no negativo CLI. Contratos,
existência do script ou --help não constituem prova MQTT/backup v16.

Comando efetivamente executado em Ubuntu26.04/WSL2, Python3.14:

```text
python3 -B -m unittest scripts.verify_core_v16_expiry.ContractTests -v
```

SHA256 do script preparado:
822318c932d73a70565402c1d307a5695fdaa1c9d36251cee2d17ffc7cbd61b3

Fixturev15 scripts/verify_core_028_state_v15.py não foi alterada; SHA256 antes
e depois debda739946a63c1fc17586cf3b6099511bee1fc492fdb4e01ae5b991f6923bb.

## Campanha real preparada, ainda sem PASS

O script exige --broker, --admin, --legacy-broker e --report explícitos,
gera estado privado em /var/tmp, limpa variáveis MQTT do ambiente herdado e
usa somente loopback/portas efêmeras/processos próprios. Não abre snapshot
ou WAL enquanto broker está ativo. Timeouts encerram apenas filhos próprios.
Não instala ferramentas nem modifica serviços, host, firewall ou hardware.

O conjunto planejado agrega nove casos: seed nativo16 com retainedTTL31,
offlineQoS1/TTL31 e Will Delay120/MessageExpiry37; backup ativo recusado;
backup/verify/restore com manifesto/bytes/hash iguais; broker15 recusando16
sem alterar snapshot/WAL; MQTT depois de restore; negativos lock/destino
existente/symlink/corrupção; seed de estado15 por ELF legado real; backup e
migração dessa cópia para16, sem inventar original ausente; MQTT migrado e
preservação da fixture15. Backup não contém marcador de perfil; somente o
marcador criado pela própria fixture é reposto na cópia restaurada.

Relatório novo guarda hashes/versões/status e valores TTL agregados, sem
payload, segredo, certificado/chave, caminho privado ou log bruto. Marker
A21_CORE_V16_EXPIRY_PASS somente após sucesso real de todo o conjunto. Não há
relatório wire neste checkpoint. Backup bruto é operação de transporte:
sozinho não prova que o admin interpreta o novo documento; origem do pin e
gates de compilação devem ser conferidos separadamente pelo coordenador.

Quando os três ELFs congelados existirem, executar com relatório novo:

```text
python3 -B scripts/verify_core_v16_expiry.py --broker /caminho/privado/broker16 --admin /caminho/privado/admin-pin16 --legacy-broker /caminho/privado/broker15 --report dist/a21-v16-new.json
```

Qualquer FAIL exige revisão do prompt antes de corrigir fonte/fixture. Não
transformar preparação em aceite global de fase6, compatibilidade publicada
ou declaração de produção. Sem push/tag/release nesta contribuição.

## Revisões v1.2/v1.3 — representação persistida, RED e GREEN reais

O coordenador identificou que hex_bytes do writer persiste payload como
string hexadecimal. A primeira check_document esperava listas de inteiros;
o selftest antigo também as construía, portanto os oito PASS iniciais eram
internos e não comprovavam compatibilidade desse oráculo com estado real.
Promptv1.2 foi registrado antes da correção no commit67b93aa.

Seed real em /var/tmp privado, Ubuntu26.04/WSL2, Python3.14, usando somente
filhos próprios, loopback/portas efêmeras, confirmou documento15 e ausência
de expiry_interval_seconds legado. Retained,offline e Will eram todos str.
check_document(documento,15,None) falhou com fixture_message_bytes_and_qos
em0.2678s. O ELF foi o P227D broker0.42.0 congelado, não broker16:

```text
broker SHA256 dca22b2624b7f36175d4e939c640b6f5730220bcc9510580eef9054a6af8ef57
admin0.10.0 SHA256 17036128ceddf37f515c3a2077bdaf57e44bd44a8cfd9f1591e9c7f848c918d3
```

Relatório RED preservado em dist/a21-v15-hex-oracle-red.json, SHA256
0b9c46cb649db40b999dcc09081665a4aa47fd5f9cd4b1804a149714235adc5a.
Ele registra o hash do harness anterior822318c9..., identidades, tipos,
document_version15/original_interval_absenttrue e hashes snapshot/WAL,
sem dump, payload, caminhos privados ou credenciais. Nenhum PASS16 emitido.

Antes de corrigir a função, novo contrato com TOML literal hex independente,
atravessando MQSNAP01/CRC32/TOML, também reproduziu1FAIL/0.005s,exit1 no
mesmo oráculo. A primeira edição teve um parêntese extra no dict aninhado
de outro selftest: SyntaxError,exit1 antes da coleta. Revisãov1.3 antecedeu
a correção somente sintática; esse erro não é uma regressão do broker.

check_payload agora exige string hex canônica minúscula/par de no máximo
8192 caracteres, decodifica e compara com os bytes sintéticos enviados.
Lista, espaço, sinal, caracteres inválidos, comprimento ímpar, excesso de
limite, bytes diferentes e None são negativos reais do novo contrato. Não
houve mudança no broker, no formato persistido ou nas quotas do produto.

Validações posteriores:

| Gate | Resultado real |
| --- | --- |
| python3 -B -m unittest scripts.verify_core_v16_expiry.ContractTests -v |9PASS,0.045s,exit0|
| Repetição seed real P227D e check_document(documento15,15,None) |PASS,0.1706s,exit0|
| git diff --check |PASS|
| Integração nativa16/migração15→16/novo admin pinado |NOT_RUN,aguarda ELFs|

A repetição usou as mesmas funções seed/document/check_document com novos
recursos temporários e relatório separado, dist/a21-v15-hex-oracle-green.json,
SHA256 a768b4965da57369662e72553fa89a34557ca5a55aff54aa5d1b51aff7b5f7bd.
Hashes snapshot diferentes entre os seeds são esperados pelas timestamps;
o WAL vazio foi idêntico. O relatório mantém explicitamente v16 NOT_RUN.
Não emite o marcador A21_CORE_V16_EXPIRY_PASS destinado ao conjunto completo.

SHA256 final do script corrigido:
b6a07b5f93260cf081b31e2b3669ab98b159156f1f4deda2a7109478386c31f8.
Fixturev15 histórica continua intocada, SHA256 debda739946a63c1fc17586cf3b6099511bee1fc492fdb4e01ae5b991f6923bb.
Os relatórios locais ignorados foram preservados; não se anexa estado bruto.
Cargo, runtime, pin, CI, versão e publicação não foram alterados por este
agente. Aguardar os três ELFs congelados para executar a fixture completa.
