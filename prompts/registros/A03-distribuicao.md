# A03 — construção e validação local antes da publicação

Data2026-10-13. PromptsA03-distribuicao v1.5 e A03-build v1.5, preparados
antes dos arquivos e revisados após falhas. GitHubpúblico criado inicialmente
vazio; isso não foi contado como código enviado, tag ou release.

A01/A02 aceitos no recorte:21testesRust,18cenáriosHTTP/MQTT e8browser; revisão
independente4casos64sockets confirmou32admitidos/32fechados, timeout10s e
recuperação200. Resultado em docs/validation.md e docs/evidence/http-admission.json.

Distribuição:9contratosPythonLinux e3contratos de exportPASS; sintaxeShellPASS.
Coletor preserva180dependências/326textos originais (superset, não SBOM doELF).
Falhas iniciaisde notices/r-efi/valuable/winapi e lifecycle com docs excluídos
registradas no A03-build. Fallbacks são explícitos/offline/commit e hashes,
sem relicenciamento; MIT própria incluída. Windowsnegativo symlink ficou
BLOCKEDporWinError1314; testes completos aprovados no alvoLinux, sem mudarSO.

Tar/deb rascunhos marcadosdraft=true passaram construção e cicloUbuntu26
descartável:install/revisão2/rollback1/remove e fixtures privadas/legacy
preservadas. ELFantesdo novolistener nesses primeirosrascunhos; não são
artefatos aceitos da release. A imagemUbuntuDocker exclui docs; lifecycle
usa path-include explícito para inspecionar notices, sem inferir defaults.
copyrightMIT adicionado antes do build exato final.

Dockerdraft Rust1.99Bookworm separado passou build e15cenários reais:
binário/version/nonroot/sinal/labels, token novo/recusaoverwrite, consultaMQTT
retained real via monitor, negativosBearer/Origin/Host/método/query/body,
headersshell, lockativo, backupofflineverifyrestore/refusadestinoexistente,
brokerindisponívelsemzeros eSIGTERMlimpo. Tudo namespace própriosemportas
publicadas e volume descartável. Não é teste de painel remoto/produção.
Dois primeirosensaiosDockerFAIL na sondaforeignHost: fetchNode24 ignoravaHost
customizado, comprovado emservidechoisolado; A03v1.5 precedeu node:http.request,
negativo403 mantido ePASS completo. Nenhuma mudança de backend por essa falha.

Gitdiffcheck e Gitleaksstaged/redact (imagem local c00b6bd0aeb3) PASS, ~250KB
de fonte, semsegredos detectados. Isso não comprova auditoria integral.
Metadados .serena/build/dist/estado não enviados. Licençacomplementar preserva
bytesGit -text; demais fontesLF, evitando conversãoWindows em clones.

Próximo gate obrigatório deste mesmoA03: commitlimpo/clonagemexata, pacotes
dessecommit/cicloexato e imagemcomlabeldessecommit, export/hashes, push/tag
imutávelv0.1.0, prerelease comassets, Hubnumérico novo/digestremoto. Estes
passos de publicação não estão inferidos pelos rascunhos; serão registrados
após execução. Broker/clientfontes/tags antigas intactos; P88documental
aguarda publicaçãoaceita. Fase3/5 não encerradas por este admin de leitura.
