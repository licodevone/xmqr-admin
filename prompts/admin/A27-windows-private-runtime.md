# A27 - arquivos privados e painel Windows nativo

Pedido autorizado: prosseguir cliente/admin Windows em paralelo, preservando A26,
Safe Rust, loopback e verificacoes de seguranca. Sem instalar servicos no host,
commit, push ou publicacao. Core Git pinado permanece imutavel.

Implementar validacao nativa de owner/DACL por handle, recusa de reparse points
em arquivo/ancestrais e concessoes amplas. Usar wrappers seguros publicados;
nenhum unsafe proprio. Token e arquivos novos recebem DACL restritiva e validacao
antes de qualquer segredo. Diretorio privado explicito e necessario. Guardar
handles de ancestrais sem compartilhamento de delete durante substituicao atomica.

Habilitar token-init/read, credenciais do console, usuarios MQTT estaticos e
bootstrap/login/estado de usuarios web no Windows. Opcao users_file no TOML
versionado resolve caminho relativo ao TOML; ausente preserva ambiente/token-only.
Nao substituir a guarda Linux do bundle dinamico no core, backup ou socket Unix.

Sincronizar SCM com broker: Stop durante inicializacao alcanca drain normal;
status inicial serializado com Stop; console aceita Ctrl+Break. Validar fmt,
testes, Clippy, ambos manifests, binario e HTTP autenticado. Registro deve distinguir
evidencia real, bloqueios de ambiente e aceite SCM ainda dependente de VM elevada.
