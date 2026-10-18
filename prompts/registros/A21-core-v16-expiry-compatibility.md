# A21 — preparação da fixture v16, integração ainda não executada

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
