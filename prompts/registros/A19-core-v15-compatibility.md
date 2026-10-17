# A19 — compatibilidade do admin com estado MQTT v15

O [prompt A19](../admin/19-core-v15-compatibility.md) precedeu a alteração do pin; rev2 corrigiu a descrição do admin publicado 0.9.0. Fonte de funcionalidade `befafc03ac20132a7e7c70b8b72b0837384d30ac`: [CI PASS](https://github.com/licodevone/xmqr-admin/actions/runs/39020123390). O marco 0.10.0 fixa o commit exato do broker `9215d4c1e6fb59f3ab73d684ecfbaf9ed38e72b2`; Cargo.lock e licenças foram atualizados. Localmente, 21 testes Rust, 3 de bundle, 3 de CLI, fmt, Clippy, build release, inventário de licenças e 11 contratos de distribuição passaram.

`scripts/verify_core_028_state_v15.py` retornou `A19_V15_PASS`: Will MQTT 5 pendente com Message Expiry persistido, backup/verify/restore offline, recuperação, lock ativo, snapshot adulterado e perfil separado. O binário publicado do admin 0.9.0 também fez backup bruto de v15, porque não interpreta o documento MQTT nessas operações; o broker publicado 0.28.0 rejeitou v15 (`Malformed`). Portanto, rollback executável do broker requer backup privado v14 anterior à primeira gravação v15. O painel do admin permanece somente leitura.

A19 integra a release 0.10.0; não houve tag isolada. Fases globais seguem abertas.
