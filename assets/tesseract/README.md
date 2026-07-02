# Bundled Tesseract OCR (Windows)

Place the following files here before building the installer:

```
assets/tesseract/
  tesseract.exe
  tessdata/
    eng.traineddata
```

On Windows, run:

```powershell
.\scripts\fetch-tesseract.ps1
```

These files are downloaded at build time and are not committed to git (see `.gitignore`).
