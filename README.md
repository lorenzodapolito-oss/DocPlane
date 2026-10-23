# DocPlane

**DocPlane** è il lanciatore del pacchetto di Lorenzo D'Apolito: una finestra con i tre programmi,
ognuno con la sua anteprima, il suo nome e a cosa serve.

| Programma | Serve per |
|---|---|
| **WordBook.it** | Videoscrittura |
| **SheetCalc.it** | Foglio di calcolo |
| **Flickdeck** | Presentazione |

![DocPlane](docs/docplane.png)

## Cosa fa

- **Un clic su un programma** lo avvia con un documento vuoto.
- **Apri file…** (su ogni scheda, oppure «Apri un altro file…» in fondo) avvia il programma giusto
  in base al tipo di file: `.docx`, `.odt`, `.rtf`… in WordBook.it; `.xlsx`, `.ods`, `.csv`… in
  SheetCalc.it; `.pptx`, `.odp`… in Flickdeck.
- **Trascina un file sulla finestra** e si apre nel programma giusto.
- **File recenti**: l'elenco dei file aperti da DocPlane (gli ultimi 12). Un clic li riapre.
- **Imposta come programmi predefiniti** (in alto a destra, solo Windows): registra WordBook.it,
  SheetCalc.it e Flickdeck per i loro tipi di file, senza bisogno di essere amministratore, e apre la
  pagina «App predefinite» di Windows. Windows 10 e 11 vogliono la conferma dell'utente per cambiare
  un programma predefinito: lì si controllano e si approvano i tre programmi.
- Se un programma non è accanto a DocPlane, la scheda dice «non trovato».

DocPlane cerca i programmi nella sua stessa cartella (`WordBook.it.exe`, `SheetCalc.it.exe`,
`Flickdeck.exe`), in una cartella con il nome del programma o in `programmi`.

## Il pacchetto per Windows

Sono tre programmi più il lanciatore, tutti nella stessa cartella:

```
DocPlane\
  DocPlane.exe
  WordBook.it.exe
  SheetCalc.it.exe
  Flickdeck.exe
  portable.txt          (SheetCalc.it: impostazioni accanto al programma)
  docplane-data\        (DocPlane)
  wordbook-data\        (WordBook.it)
  flickdeck-data\       (Flickdeck)
```

Con le cartelle `*-data` e il file `portable.txt` tutto è portatile: impostazioni e file recenti
restano nella cartella, che si può mettere su una chiavetta. Cancellali per usare il profilo di
Windows.

### Come crearlo

Servono Rust (toolchain `x86_64-pc-windows-gnu`) e un compilatore C come
[WinLibs](https://winlibs.com) (UCRT, 64 bit), **in una cartella senza spazi nel percorso**, e Git.
Poi, nel Prompt dei comandi, dalla cartella di DocPlane:

```
packaging\windows\Crea-pacchetto.bat
```

Lo script scarica o aggiorna i tre programmi (cartelle accanto a quella di DocPlane), li compila
e crea `dist\DocPlane\` e `dist\DocPlane.zip`. La prima volta richiede circa un'ora.

Solo DocPlane, senza il resto: `cargo build --release -p docplane`.

## Licenza

MIT OR Apache-2.0. L'icona e le anteprime sono disegnate nel codice e sono originali. Vedi
[ATTRIBUTION.md](ATTRIBUTION.md).
