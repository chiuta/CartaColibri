.PHONY: all glm deepseek-v4 portable test check cuda-test clean install uninstall

all glm deepseek-v4 portable test check cuda-test clean install uninstall:
	$(MAKE) -C c $@
# CartaColibri (RAG-fetch + RAG-CARTA): pure-Python tests, no model needed
.PHONY: check-carta
check-carta:
	python3 -I tests/test_cartacolibri.py
