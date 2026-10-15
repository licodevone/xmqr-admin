# A09 rev1 — exportar e rotacionar auditoria administrativa

2026-10-15. Entrada admin `a007307`, broker `fc4ebb1` com P111 em andamento.
Auditoria consultável mas 128 recibos não podem ser apagados sem defesa
contra replay. Esta fatia acompanha o prompt broker P111; o core Git do admin
permanece pinado.

CLI `channel audit-rotate --archive NOVO_ARQUIVO` consulta páginas operator,
monta JSON canônico, grava arquivo privado 0600 exclusivo, sincroniza arquivo
e diretório, calcula SHA256 e só então solicita rotação com hash/contagem.
`audit-verify --file ARQUIVO` verifica conteúdo exportado offline. Após rotação,
novos IDs de mutação codificam época do broker, lida de status; IDs antigos
recusados pelo ator. Em timeout, mostrar ID para consultar receipt, sem retry
automático. Não incluir token, payload, senha ou identificadores de alvo.

Negativos: arquivo existente/público/symlink, checksum alterado, mudança de
recibos entre export e commit, ID de época antiga, falha WAL e restart.
Integração real Linux/WSL com broker isolado; gates fmt/test/Clippy/build,
licenças e docs antes de aceitar A05/A09. O painel web continua leitura.
