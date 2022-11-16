Invoke-Command -Script { cargo run } -ErrorAction SilentlyContinue
IF( $LASTEXITCODE -EQ 5 ) {
  Write-Output "eduOS-rs runs succesfully within Qemu"
  Exit 0
} else {
  Write-Output echo "eduOS-rs isn't able to run within Qemu"
  Exit 1
}
