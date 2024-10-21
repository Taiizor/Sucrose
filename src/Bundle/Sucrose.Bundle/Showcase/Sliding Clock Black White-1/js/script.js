function SucrosePropertyListener(name, val) {
	switch (name) {
		case "themeMode":
			switch (val.value) {
				case 0:
					//Dark
					themeMode(0);
					break;
				case 1:
					//Light
					themeMode(1);
					break;
				default:
					break;
			}
			break;
		case "themeChange":
			themeChange(val.value)
			break;
		default:
			break;
	}
}