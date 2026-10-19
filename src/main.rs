// SPDX-License-Identifier: MIT
mod bundle;
mod bundle_edit;
mod channel;
mod cli;
mod config;
mod listener;
mod monitor;
mod mqtt_console;
mod state;
mod token;
mod users;
mod web;
mod web_auth;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.as_slice() {
        [group, command, flag, path]
            if group == "bundle" && command == "validate" && flag == "--file" =>
        {
            bundle::validate(std::path::Path::new(path))?;
            println!("Bundle seguro valido. Nenhum arquivo foi alterado.");
            Ok(())
        }
        [group, ..] if group == "bundle" => {
            bundle_edit::run(&std::env::args_os().skip(1).collect::<Vec<_>>())?;
            Ok(())
        }
        [group, ..] if group == "channel" => {
            channel::run(&args)?;
            Ok(())
        }
        [command, flag, path] if command == "audit-verify" && flag == "--file" => {
            channel::verify_archive(std::path::Path::new(path))?;
            Ok(())
        }
        [command, ..] if command == "diagnose" => {
            channel::diagnose(&args)?;
            Ok(())
        }
        [command] if command == "serve" => {
            let config = config::Config::from_env()?;
            let token = token::read(&config.token_file)?;
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?
                .block_on(web::serve(config, token))?;
            Ok(())
        }
        [command, flag, path] if command == "token-init" && flag == "--file" => {
            token::create(std::path::Path::new(path))?;
            println!("Token local criado em arquivo privado. Nenhum valor foi exibido.");
            Ok(())
        }
        [command] if command == "--help" || command == "-h" => {
            println!(
                "XMQR Admin {}\n\nPainel: xmqr-admin serve\nToken: xmqr-admin token-init --file NOVO_ARQUIVO\nUsuários: user create/list/update-password/delete\nEstado: state backup/verify/restore\nBundle seguro: bundle validate --file ARQUIVO; bundle role|group list/create/update/delete --file ARQUIVO; bundle user list/enable/disable/update-acl/set-membership --file ARQUIVO\nCanal local: channel status|receipt|reload|sessions|retained|retained-delete|session-disconnect|audit|audit-rotate --socket CAMINHO --token-file ARQUIVO [opcoes]\nDiagnóstico: diagnose --state-dir DIR --bundle ARQUIVO --backup ARQUIVO --socket CAMINHO --token-file ARQUIVO\nAuditoria offline: audit-verify --file ARQUIVO\nSegurança: hash-password; fingerprint CERTIFICADO\nVersão: --version\n\nConsulte README para argumentos completos. Painel local somente leitura.",
                env!("CARGO_PKG_VERSION")
            );
            Ok(())
        }
        _ => cli::run(),
    }
}
