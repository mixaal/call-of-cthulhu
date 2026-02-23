for img in *.png; do tesseract "$img" "${img%.png}" -l ces ; done
