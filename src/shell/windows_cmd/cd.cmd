@echo off
cd %*
if "%PACTUP_VERSION_FILE_STRATEGY%" == "recursive" (
  pactup use --silent-if-unchanged
) else (
  if exist .pactrc (
    pactup use --silent-if-unchanged
  ) else (
    if exist .pact-version (
      pactup use --silent-if-unchanged
    )
  )
)
@echo on
