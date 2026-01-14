.PHONY: build-release
build-release:
	cargo build --release

.PHONY: install
install:
	cargo install --path=. --force --root=/usr
	cp -r etc/xdg/* /etc/xdg/
	cp pwm.desktop /usr/share/xsessions/

.PHONY: clean
clean:
	rm -rf target

.PHONY: uninstall
uninstall:
	rm /usr/bin/pwm
	rm -rf /etc/xdg/pwm
	rm /usr/share/xsessions/pwm.desktop
