# A08 rev1 — CLI independente para o canal administrativo

2026-10-15. Entrada: admin 0.3.0 (`80b90ea`), broker `7c25e3f` com
`status`, `reload` e `receipt` no socket Unix P107. O editor de bundle é
offline; a fase 5 global e A09 ainda estão abertos. O core Git permanece
pinado no commit existente, sem dependência por caminho.

Implementar `channel status|receipt|reload` no `xmqr-admin`. A CLI lê token
somente de arquivo privado, confirma socket local privado, limita request e
response, aplica prazo de I/O e nunca exibe token. `reload` valida bundle
antes da rede, calcula SHA256 dos bytes privados e envia geração esperada e
ID aleatório de 128 bits. Exibir o ID antes do envio e aceitar ID fornecido
para recuperação; timeout/EOF/resultado desconhecido instrui consulta de
`receipt` com o mesmo ID, sem repetição automática. Não abrir WAL nem
substituir o editor offline.

Arquivos: `src/main.rs`, novo módulo `src/channel.rs`, `src/token.rs`, testes,
README e registro A08. Não colocar token em argumentos nem JSON de saída.
Negativos: socket/token públicos ou symlink, token errado, viewer sem mutação,
ID inválido, geração inválida, bundle alterado, request/resposta grandes,
conexão lenta e resposta sem correlação. Provar integração com processo broker
isolado, restart e recibo; não usar serviços/estado reais. Gates fmt, test,
Clippy, build release e inventário de licenças no Ubuntu/WSL.

Compatibilidade: protocolo JSON v1 P107, Linux/WSL, sem alterar o pin do core
ou o painel web. A09 só será concluído após operações de sessão/retained e
testes de falha/concorrência; esta CLI é uma fatia parcial.

## rev2 — inventários paginados P108

Expor `channel sessions` e `channel retained` usando token operator. Aceitar
`--limit` e `--cursor` opcionais, com limites do broker (16/8 itens), imprimir
JSON de metadados sem payload e cursor seguinte. Não inferir snapshot estável
entre páginas sob MQTT concorrente. Validar integração com broker isolado,
viewer negado e limites; A07 continua parcial até as ações auditadas.

## rev3 — remover retained com recibo

Expor `channel retained-delete --topic TOPICO [--request-id ID]` somente com
token operator. Mostrar request_id antes de enviar, não mostrar payload nem
repetir em timeout. Recusa de tópico inválido, viewer e ID conflitante;
consulta de recibo recupera resultado após restart. O broker P108 usa documento
MQTT v4; documentar rollback e não prometer ação de sessão nesta fatia.

## rev4 — desconectar sessão ativa

Expor `channel session-disconnect --client-id ID [--request-id ID]`, com token
operator e ID recuperável. Broker publica Will autorizado como desconexão não
graciosa, preservando sessão persistente e inbound QoS2 já aceito; a CLI não
abre o WAL. Testar viewer proibido, sessão ausente, replay pós-restart e
conexão real encerrada. Não chamar ação de exclusão de sessão persistente.

## rev5 — consulta limitada de auditoria

Expor `channel audit` para operator, com `--limit` 1..16 e `--cursor`
numérico opcional. Imprimir somente metadados sem segredo, sem abrir WAL.
Provar viewer negado, duas páginas, restart e ausência de payload/senha.
Retenção/rotação de auditoria ainda exige decisão separada; não marcar A05
como completo só pela listagem.
