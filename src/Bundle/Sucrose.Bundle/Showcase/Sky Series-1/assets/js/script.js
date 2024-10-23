SucrosePropertyListener = function(name, val) {
	switch (name) {
		case "bgImage":
			document.body.style.backgroundImage = 'url("./' + val.folder + '/' + val.value + '")';
			break;
	}
}