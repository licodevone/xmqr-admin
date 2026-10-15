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
