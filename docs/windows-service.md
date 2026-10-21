> Historico A26: o bloqueio de token descrito abaixo foi resolvido pelo [runtime nativo A27](windows-native-runtime.md). Consulte A27 para o contrato atual e evidencias.

# Serviço Windows - A26 (experimental, aceite pendente)

O painel recebe um adaptador opt-in para o Service Control Manager:
O manifest experimental `packaging/windows-service/Cargo.toml` habilita a
feature `windows-service-runtime` sobre a fonte existente. O manifest principal
preserva dependências, lock e a CLI normal. Dependência, lock experimental e
build debug Windows x64 MSVC comprovados em 2026-10-21, com marker real e quatro
testes dirigidos aprovados. Clippy estrito ainda falha nos avisos Windows
preexistentes; o negativo de token confirmou a recusa nativa abaixo.
Veja o [registro A26](../prompts/registros/A26-windows-service.md).
O build normal não aceita o prefixo de serviço.

```text
xmqr-admin.exe --windows-service xmqr-admin "C:\ProgramData\XMQR\admin\logs" --config "C:\ProgramData\XMQR\admin\admin.toml"
```

Configuração de endereços, sem segredos:

```toml
version = 1
bind = "127.0.0.1:9080"
monitor = "127.0.0.1:9090"
token_file = "token"
```

`token_file` relativo é resolvido a partir da pasta do TOML. O caminho passado
em `--config` deve ser absoluto. O arquivo é limitado a 64 KiB, com versão
obrigatória e recusa de campos desconhecidos. Bind e monitor continuam aceitando
somente IP loopback e porta diferente de zero. `serve --config CAMINHO` permite
usar o mesmo arquivo em console; `serve` sem opção preserva `XMQR_ADMIN_*`.
As opções existentes de autenticação e console MQTT continuam sob seus próprios
contratos; o arquivo acima não guarda tokens, senhas nem configuração de usuários.

**Bloqueio mantido:** token privado exige Unix; não há validação equivalente
de ACL Windows nesta etapa. O serviço falha antes de reportar Running. Não
contornar copiando segredo para arquivo público ou removendo a verificação.
O core Git pinado, os formatos de estado e os comandos existentes são preservados.

Após os pré-requisitos de plataforma, o ciclo SCM usa StartPending/Running,
StopPending e Stopped. Stop/Shutdown encaminham para a parada HTTP e do console
MQTT existente. Falha de inicialização/runtime resulta em exit code não zero no
SCM e registro fixo em `service.log` (sem segredos, até 1 MiB mais arquivo anterior).
Running não é evidência de que o broker remoto está pronto.

A bandeja e os pacotes Windows compartilhados ficam no repositório do broker.
Nenhuma instalação, permissão do host ou versão publicada foi alterada.
Veja [prompt](../prompts/admin/A26-windows-service.md) e
[registro](../prompts/registros/A26-windows-service.md).
