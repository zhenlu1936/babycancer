.PHONY: all clean rebuild

# Main target - builds the PDF with proper bibliography processing
all:
	xelatex -synctex=1 -interaction=nonstopmode presentation.tex
	biber presentation
	xelatex -synctex=1 -interaction=nonstopmode presentation.tex

# Clean all generated files
clean:
	latexmk -C
	rm -f *.aux *.bbl *.bcf *.blg *.nav *.run.xml *.snm *.toc *.vrb *.log *.out *.fls *.fdb_latexmk *.synctex.gz

# Complete rebuild (clean then build)
rebuild: clean all
