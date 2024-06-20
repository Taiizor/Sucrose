window.onload = function () {
    //MARK: Update
    const version = "v7.3.0";

    checkForUpdates = async () => {
        const url = 'https://api.github.com/repos/IPdotSetAF/NeoMatrix/tags';
        const tags = await fetch(url).then(_ => _.json());
        if (tags[0]['name'] > version)
            Log("New release available: " + tags[0]['name']);
    }

    readProjectConfig = async () => {
        return await fetch('project.json').then(_ => _.json());
    }

    //MARK: Options
    var gui;
    var options = {
        ui_rain_matrixSpeed: 24,
        fpsInterval: calculateFpsInterval(24),
        ui_rain_trailLength: 0.86,
        trailLength: calculateTrailLength(0.86),
        ui_rain_dropCount: 1,
        ui_rain_initialAnimation: "1",
        ui_characters_charset: "4",
        ui_characters_customCharset: "0123456789ABCDEF",
        ui_font_font: "3",
        ui_font_customFont: "monospace",
        ui_font_size: 15,
        ui_other_codesCommaSeparated: "IP.AF,THE MATRIX",
        codes: makeCodes("IP.AF,THE MATRIX"),
        ui_color_colorMode: "2",
        ui_color_matrixColor: [0, 1, 0],
        matrixColor: rgbToHue([0, 1, 0]),
        ui_color_colorAnimationSpeed: 0.5,
        colorAnimationSpeed: calculateColorAnimationSpeed(0.5),
        ui_color_highlightFirstCharacter: true,
        ui_audio_audioResponsive: false,
        ui_audio_audioSensetivity: 50,
        ui_audio_silenceAnimation: true,
        ui_audio_silenceTimeoutSeconds: 3,
        ui_logo_logo: "0",
        ui_logo_customLogo: "",
        ui_logo_preserveColor: false,
        ui_logo_scale: 1,
        ui_logo_positionX: 0,
        ui_logo_positionY: 0,
        ui_clock_clock: "0",
        ui_clock_24HourFormat: true,
        ui_clock_dayLightSaving: 0,
        ui_clock_scale: 1,
        ui_clock_positionX: 0,
        ui_clock_positionY: 0,
        ui_message_message: false,
        ui_message_text: "THE MATRIX",
        ui_message_scale: 1,
        ui_message_positionX: 0,
        ui_message_positionY: 0,
        ui_day_day: "0",
        ui_day_allCaps: false,
        ui_day_orientation: false,
        ui_day_scale: 1,
        ui_day_positionX: 0,
        ui_day_positionY: 0,
        ui_date_date: "0",
        ui_date_style: "0",
        ui_date_year: "2",
        ui_date_order: "0",
        ui_date_monthName: false,
        ui_date_allCaps: false,
        ui_date_delimiter: "0",
        ui_date_scale: 1,
        ui_date_positionX: 0,
        ui_date_positionY: 0,
        Share() {
            copyToClipboard(paramsToUrl({ preset: btoa(JSON.stringify(gui.save())) }, {}, []));
            Log("Copied Preset URL to clipboard.");
        },
        Save() {
            window.localStorage.setItem("preset", JSON.stringify(gui.save()));
            Log("Saved preset.");
        },
        Load() {
            let preset = JSON.parse(window.localStorage.getItem("preset"));
            if (preset) {
                gui.load(preset);
                Log("Loaded preset.");
            } else
                Log("No preset found.");
        },
        Reset() {
            gui.reset();
            Log("Settings reset to default.");
        },
        LoadFrom(params) {
            let preset = gui.load(JSON.parse(atob(params.preset)));
            if (preset) {
                gui.load(preset);
                Log("Loaded preset from URL.");
            } else
                Log("Preset URl is not correct.");
        }
    }

    if (window.wallpaperRegisterAudioListener)
        window.wallpaperRegisterAudioListener((audioArray) => {
            return frequencyArray = audioArray;
        });
    else if (navigator.userAgent.startsWith("Sucrose"))
        window.SucroseAudioData = function (audioArray) {
            frequencyArray = audioArray.Data;
        };
    else
        drawGui();

    //MARK: GUI
    function drawGui() {
        const params = getUrlParams();

        readProjectConfig().then((config) => {
            gui = new lil.GUI({ autoPlace: false, width: 300 });

            const rainFolder = gui.addFolder('Rain');
            rainFolder.add(options, 'ui_rain_matrixSpeed').min(1).max(60).step(1).name('Matrix Speed').onChange(() => {
                options.fpsInterval = calculateFpsInterval(options.ui_rain_matrixSpeed);
            });
            rainFolder.add(options, 'ui_rain_trailLength').min(0).max(1).step(0.01).name('Trail Length').onChange(() => {
                options.trailLength = calculateTrailLength(options.ui_rain_trailLength);
                updateMask();
            });
            rainFolder.add(options, "ui_rain_dropCount").min(1).max(5).step(1).name("Drop Count/Column").onChange(initialAnimation);
            rainFolder.add(options, "ui_rain_initialAnimation", optionsToDict(config.general.properties.ui_rain_initialanimation.options)).name("Initial Animation").onChange(initialAnimation);
            rainFolder.close();

            const colorFolder = gui.addFolder("Color");
            colorFolder.add(options, 'ui_color_colorMode', optionsToDict(config.general.properties.ui_color_colormode.options)).name('Color Mode');
            colorFolder.addColor(options, 'ui_color_matrixColor').name('Matrix Color').onChange(() => {
                options.matrixColor = rgbToHue(options.ui_color_matrixColor);
            });
            colorFolder.add(options, 'ui_color_colorAnimationSpeed').min(-1).max(1).step(0.01).name('Color Animation Speed').onChange(() => {
                options.colorAnimationSpeed = calculateColorAnimationSpeed(options.ui_color_colorAnimationSpeed);
            });
            colorFolder.add(options, 'ui_color_highlightFirstCharacter').name('Highlight First Character');
            colorFolder.close();

            const characterFolder = gui.addFolder("Characters");
            characterFolder.add(options, 'ui_characters_charset', optionsToDict(config.general.properties.ui_characters_charset.options)).name('Char set').onChange(updateCharSet);
            characterFolder.add(options, 'ui_characters_customCharset').name('Custom Char Set').onChange(updateCharSet);
            characterFolder.close();

            const fontFolder = gui.addFolder("Font");
            fontFolder.add(options, 'ui_font_size').min(5).max(30).step(1).name('Font Size').onChange(updateFont);
            fontFolder.add(options, 'ui_font_font', optionsToDict(config.general.properties.ui_font_font.options)).name('Font').onChange(updateFont);
            fontFolder.add(options, 'ui_font_customFont').name('Custom Font').onChange(updateFont);
            fontFolder.close();

            gui.addFolder("Audio (not available in web version)").close();;

            const logoFolder = gui.addFolder("Logo");
            logoFolder.add(options, "ui_logo_logo", optionsToDict(config.general.properties.ui_logo_logo.options)).name("Logo").onChange(updateLogo);
            logoFolder.add(options, "ui_logo_customLogo").name("Custom Logo URL (SVG/PNG)").onChange(updateLogo);
            logoFolder.add(options, "ui_logo_preserveColor").name("Preserve Logo Color").onChange(updateLogo);
            logoFolder.add(options, "ui_logo_scale").min(0).max(10).step(0.1).name("Scale").onChange(updateLogo);
            const logoPositionFolder = logoFolder.addFolder("Position");
            logoPositionFolder.add(options, "ui_logo_positionX").min(-2500).max(2500).step(1).name("X").onChange(updateLogo);
            logoPositionFolder.add(options, "ui_logo_positionY").min(-2500).max(2500).step(1).name("Y").onChange(updateLogo);
            logoFolder.close();

            const clockFolder = gui.addFolder("Clock");
            clockFolder.add(options, "ui_clock_clock", optionsToDict(config.general.properties.ui_clock_clock.options)).name("Clock").onChange(updateMask);
            clockFolder.add(options, "ui_clock_24HourFormat").name("24 Hour format").onChange(() => {
                updateTime();
                updateMask();
            });
            clockFolder.add(options, "ui_clock_dayLightSaving").min(-1).max(1).step(1).name("Day-light Saving").onChange(() => {
                updateTime();
                updateMask();
            });
            clockFolder.add(options, "ui_clock_scale").min(0).max(10).step(1).name("Scale").onChange(updateMask);
            const clockPositionFolder = clockFolder.addFolder("Position");
            clockPositionFolder.add(options, "ui_clock_positionX").min(-100).max(100).step(1).name("X").onChange(updateMask);
            clockPositionFolder.add(options, "ui_clock_positionY").min(-100).max(100).step(1).name("Y").onChange(updateMask);
            clockFolder.close();

            const dayFolder = gui.addFolder("Day");
            dayFolder.add(options, "ui_day_day", optionsToDict(config.general.properties.ui_day_day.options)).name("Day").onChange(updateMask);
            dayFolder.add(options, "ui_day_allCaps").name("All CAPS").onChange(updateMask);
            dayFolder.add(options, "ui_day_orientation").name("Vertical Orientation").onChange(updateMask);
            dayFolder.add(options, "ui_day_scale").min(0).max(10).step(1).name("Scale").onChange(updateMask);
            const dayPositionFolder = dayFolder.addFolder("Position");
            dayPositionFolder.add(options, "ui_day_positionX").min(-100).max(100).step(1).name("X").onChange(updateMask);
            dayPositionFolder.add(options, "ui_day_positionY").min(-100).max(100).step(1).name("Y").onChange(updateMask);
            dayFolder.close();

            const dateFolder = gui.addFolder("Date");
            dateFolder.add(options, "ui_date_date", optionsToDict(config.general.properties.ui_date_date.options)).name("Date").onChange(() => {
                updateTime();
                updateMask();
            });
            dateFolder.add(options, "ui_date_year", optionsToDict(config.general.properties.ui_date_year.options)).name("Year").onChange(updateMask);
            dateFolder.add(options, "ui_date_order", optionsToDict(config.general.properties.ui_date_order.options)).name("Order").onChange(updateMask);
            dateFolder.add(options, "ui_date_monthName").name("Month Name").onChange(updateMask);
            dateFolder.add(options, "ui_date_allCaps").name("All CAPS").onChange(updateMask);
            dateFolder.add(options, "ui_date_delimiter", optionsToDict(config.general.properties.ui_date_delimiter.options)).name("Delimiter").onChange(updateMask);
            dateFolder.add(options, "ui_date_style", optionsToDict(config.general.properties.ui_date_style.options)).name("Style").onChange(updateMask);
            dateFolder.add(options, "ui_date_scale").min(0).max(10).step(1).name("Scale").onChange(updateMask);
            const datePositionFolder = dateFolder.addFolder("Position");
            datePositionFolder.add(options, "ui_date_positionX").min(-100).max(100).step(1).name("X").onChange(updateMask);
            datePositionFolder.add(options, "ui_date_positionY").min(-100).max(100).step(1).name("Y").onChange(updateMask);
            dateFolder.close();

            const messageFolder = gui.addFolder("Message");
            messageFolder.add(options, "ui_message_message").name("Message").onChange(updateMask);
            messageFolder.add(options, "ui_message_text").name("Message Text").onChange(updateMask);
            messageFolder.add(options, "ui_message_scale").min(0).max(10).step(1).name("Scale").onChange(updateMask);
            const messagePositionFolder = messageFolder.addFolder("Position");
            messagePositionFolder.add(options, "ui_message_positionX").min(-100).max(100).step(1).name("X").onChange(updateMask);
            messagePositionFolder.add(options, "ui_message_positionY").min(-100).max(100).step(1).name("Y").onChange(updateMask);
            messageFolder.close();

            const otherFolder = gui.addFolder("Other");
            otherFolder.add(options, 'ui_other_codesCommaSeparated').name('Codes (Comma separated)').onChange(() => {
                options.codes = makeCodes(options.ui_other_codesCommaSeparated);
                initialAnimation();
            });
            otherFolder.close();

            gui.add(options, "Share");
            gui.add(options, "Save");
            gui.add(options, "Load");
            gui.add(options, "Reset");

            customContainer = document.getElementById('gui');
            customContainer.appendChild(gui.domElement);

            if (params)
                options.LoadFrom(params);
            else
                options.Load();
        });
    }

    //MARK: Wallpaper Engine
    window.wallpaperPropertyListener = {
        applyUserProperties: function (properties) {
            if (properties.ui_rain_matrixspeed)
                options.fpsInterval = calculateFpsInterval(properties.ui_rain_matrixspeed.value);
            if (properties.ui_rain_traillength) {
                options.trailLength = calculateTrailLength(properties.ui_rain_traillength.value);
                updateMask();
            }
            if (properties.ui_rain_initialanimation)
                options.ui_rain_initialAnimation = properties.ui_rain_initialanimation.value;
            if (properties.ui_rain_dropcount)
                options.ui_rain_dropCount = properties.ui_rain_dropcount.value;
            if (properties.ui_rain_initialanimation || properties.ui_rain_dropcount)
                initialAnimation();

            if (properties.ui_color_colormode)
                options.ui_color_colorMode = properties.ui_color_colormode.value;
            if (properties.ui_color_matrixcolor)
                options.matrixColor = rgbToHue(properties.ui_color_matrixcolor.value.split(' '))
            if (properties.ui_color_coloranimationspeed)
                options.colorAnimationSpeed = calculateColorAnimationSpeed(properties.ui_color_coloranimationspeed.value);
            if (properties.ui_color_highlightfirstcharacter)
                options.ui_color_highlightFirstCharacter = properties.ui_color_highlightfirstcharacter.value;

            if (properties.ui_characters_charset)
                options.ui_characters_charset = properties.ui_characters_charset.value;
            if (properties.ui_characters_customcharset)
                options.ui_characters_customCharset = properties.ui_characters_customcharset.value;
            if (properties.ui_characters_charset || properties.ui_characters_customcharset)
                updateCharSet();

            if (properties.ui_font_font)
                options.ui_font_font = properties.ui_font_font.value;
            if (properties.ui_font_customFont)
                options.ui_font_customFont = properties.ui_font_customFont.value;
            if (properties.ui_font_size)
                options.ui_font_size = properties.ui_font_size.value;
            if (properties.ui_font_font || properties.ui_font_customFont || properties.ui_font_size)
                updateFont();

            if (properties.ui_audio_audioresponsive)
                options.ui_audio_audioResponsive = properties.ui_audio_audioresponsive.value;
            if (properties.ui_audio_audiosensetivity)
                options.ui_audio_audioSensetivity = properties.ui_audio_audiosensetivity.value;
            if (properties.ui_audio_silenceanimation)
                options.ui_audio_silenceAnimation = properties.ui_audio_silenceanimation.value;
            if (properties.ui_audio_silencetimeoutseconds)
                options.ui_audio_silenceTimeoutSeconds = properties.ui_audio_silencetimeoutseconds.value;

            if (properties.ui_logo_logo)
                options.ui_logo_logo = properties.ui_logo_logo.value;
            if (properties.ui_logo_customlogo)
                options.ui_logo_customLogo = properties.ui_logo_customlogo.value;
            if (properties.ui_logo_scale)
                options.ui_logo_scale = properties.ui_logo_scale.value;
            if (properties.ui_logo_positionx)
                options.ui_logo_positionX = properties.ui_logo_positionx.value;
            if (properties.ui_logo_positiony)
                options.ui_logo_positionY = properties.ui_logo_positiony.value;
            if (properties.ui_logo_preservecolor)
                options.ui_logo_preserveColor = properties.ui_logo_preservecolor.value;
            if (properties.ui_logo_logo || properties.ui_logo_customlogo || properties.ui_logo_scale ||
                properties.ui_logo_positionx || properties.ui_logo_positiony || properties.ui_logo_preservecolor)
                updateLogo();

            if (properties.ui_clock_clock)
                options.ui_clock_clock = properties.ui_clock_clock.value;
            if (properties.ui_clock_24hourformat) {
                options.ui_clock_24HourFormat = properties.ui_clock_24hourformat.value;
                updateTime();
            }
            if (properties.ui_clock_daylightsaving) {
                options.ui_clock_dayLightSaving = properties.ui_clock_daylightsaving.value;
                updateTime();
            }
            if (properties.ui_clock_scale)
                options.ui_clock_scale = properties.ui_clock_scale.value;
            if (properties.ui_clock_positionx)
                options.ui_clock_positionX = properties.ui_clock_positionx.value;
            if (properties.ui_clock_positiony)
                options.ui_clock_positionY = properties.ui_clock_positiony.value;
            if (properties.ui_clock_clock || properties.ui_clock_24hourformat || properties.ui_clock_daylightsaving ||
                properties.ui_clock_scale || properties.ui_clock_positionx || properties.ui_clock_positiony)
                updateMask();

            if (properties.ui_day_day)
                options.ui_day_day = properties.ui_day_day.value;
            if (properties.ui_day_allcaps)
                options.ui_day_allCaps = properties.ui_day_allcaps.value;
            if (properties.ui_day_orientation)
                options.ui_day_orientation = properties.ui_day_orientation.value;
            if (properties.ui_day_scale)
                options.ui_day_scale = properties.ui_day_scale.value;
            if (properties.ui_day_positionx)
                options.ui_day_positionX = properties.ui_day_positionx.value;
            if (properties.ui_day_positiony)
                options.ui_day_positionY = properties.ui_day_positiony.value;
            if (properties.ui_day_day || properties.ui_day_allcaps || properties.ui_day_orientation ||
                properties.ui_day_scale || properties.ui_day_positionx || properties.ui_day_positiony)
                updateMask();

            if (properties.ui_date_date) {
                options.ui_date_date = properties.ui_date_date.value;
                updateTime();
            }
            if (properties.ui_date_style)
                options.ui_date_style = properties.ui_date_style.value;
            if (properties.ui_date_year)
                options.ui_date_year = properties.ui_date_year.value;
            if (properties.ui_date_order)
                options.ui_date_order = properties.ui_date_order.value;
            if (properties.ui_date_monthname)
                options.ui_date_monthName = properties.ui_date_monthname.value;
            if (properties.ui_date_allcaps)
                options.ui_date_allCaps = properties.ui_date_allcaps.value;
            if (properties.ui_date_delimiter)
                options.ui_date_delimiter = properties.ui_date_delimiter.value;
            if (properties.ui_date_scale)
                options.ui_date_scale = properties.ui_date_scale.value;
            if (properties.ui_date_positionx)
                options.ui_date_positionX = properties.ui_date_positionx.value;
            if (properties.ui_date_positiony)
                options.ui_date_positionY = properties.ui_date_positiony.value;
            if (properties.ui_date_date || properties.ui_date_style || properties.ui_date_year ||
                properties.ui_date_order || properties.ui_date_monthname || properties.ui_date_allcaps ||
                properties.ui_date_delimiter || properties.ui_date_scale || properties.ui_date_positionx
                || properties.ui_date_positiony)
                updateMask();

            if (properties.ui_message_message)
                options.ui_message_message = properties.ui_message_message.value;
            if (properties.ui_message_text)
                options.ui_message_text = properties.ui_message_text.value;
            if (properties.ui_message_scale)
                options.ui_message_scale = properties.ui_message_scale.value;
            if (properties.ui_message_positionx)
                options.ui_message_positionX = properties.ui_message_positionx.value;
            if (properties.ui_message_positiony)
                options.ui_message_positionY = properties.ui_message_positiony.value;
            if (properties.ui_message_message || properties.ui_message_text || properties.ui_message_scale ||
                properties.ui_message_positionx || properties.ui_message_positiony)
                updateMask();

            if (properties.ui_other_codescommaseparated) {
                options.codes = makeCodes(properties.ui_other_codescommaseparated.value);
                initialAnimation();
            }
        }
    };

    //MARK: Sucrose Wallpaper Engine
    window.SucrosePropertyListener = function (name, val) {
        switch (name) {
            case "ui_rain_matrixspeed":
                options.fpsInterval = calculateFpsInterval(val.value);
                break;
            case "ui_rain_traillength":
                options.trailLength = calculateTrailLength(val.value / 100);
                updateMask();
                break;
            case "ui_rain_initialanimation":
                options.ui_rain_initialAnimation = val.value.toString();
                initialAnimation();
                break;
            case "ui_rain_dropcount":
                options.ui_rain_dropCount = val.value;
                initialAnimation();
                break;

            case "ui_color_colormode":
                options.ui_color_colorMode = val.value.toString();
                break;
            case "ui_color_matrixcolor":
                const tmp = hexToRgb(val.value);
                options.matrixColor = rgbToHue([tmp.r, tmp.g, tmp.b])
                break;
            case "ui_color_coloranimationspeed":
                options.colorAnimationSpeed = calculateColorAnimationSpeed(val.value / 10);
                break;
            case "ui_color_highlightfirstcharacter":
                options.ui_color_highlightFirstCharacter = val.value;
                break;

            case "ui_characters_charset":
                options.ui_characters_charset = val.value.toString();
                updateCharSet();
                break;
            case "ui_characters_customcharset":
                options.ui_characters_customCharset = val.value;
                updateCharSet();
                break;

            case "ui_font_font":
                options.ui_font_font = val.value.toString();
                updateFont();
                break;
            case "ui_font_customfont":
                options.ui_font_customFont = val.value;
                updateFont();
                break;
            case "ui_font_size":
                options.ui_font_size = val.value;
                updateFont();
                break;

            case "ui_audio_audioresponsive":
                options.ui_audio_audioResponsive = val.value;
                break;
            case "ui_audio_audiosensetivity":
                options.ui_audio_audioSensetivity = val.value;
                break;
            case "ui_audio_silenceanimation":
                options.ui_audio_silenceAnimation = val.value;
                break;
            case "ui_audio_silencetimeoutseconds":
                options.ui_audio_silenceTimeoutSeconds = val.value;
                break;

            case "ui_logo_logo":
                options.ui_logo_logo = val.value.toString();
                updateLogo();
                break;
            case "ui_logo_customlogo":
                options.ui_logo_customLogo = val.value;
                updateLogo();
                break;
            case "ui_logo_preservecolor":
                options.ui_logo_preserveColor = val.value;
                updateLogo();
                break;
            case "ui_logo_scale":
                options.ui_logo_scale = val.value / 10;
                updateLogo();
                break;
            case "ui_logo_positionx":
                options.ui_logo_positionX = val.value;
                updateLogo();
                break;
            case "ui_logo_positiony":
                options.ui_logo_positionY = val.value;
                updateLogo();
                break;

            case "ui_clock_clock":
                options.ui_clock_clock = val.value.toString();
                updateMask();
                break;
            case "ui_clock_24hourformat":
                options.ui_clock_24HourFormat = val.value;
                updateTime();
                updateMask();
                break;
            case "ui_clock_daylightsaving":
                options.ui_clock_dayLightSaving = val.value;
                updateTime();
                updateMask();
                break;
            case "ui_clock_scale":
                options.ui_clock_scale = val.value;
                updateMask();
                break;
            case "ui_clock_positionx":
                options.ui_clock_positionX = val.value;
                updateMask();
                break;
            case "ui_clock_positiony":
                options.ui_clock_positionY = val.value;
                updateMask();
                break;

            case "ui_day_day":
                options.ui_day_day = val.value.toString();
                updateMask();
                break;
            case "ui_day_allcaps":
                options.ui_day_allCaps = val.value;
                updateMask();
                break;
            case "ui_day_orientation":
                options.ui_day_orientation = val.value;
                updateMask();
                break;
            case "ui_day_scale":
                options.ui_day_scale = val.value;
                updateMask();
                break;
            case "ui_day_positionx":
                options.ui_day_positionX = val.value;
                updateMask();
                break;
            case "ui_day_positiony":
                options.ui_day_positionY = val.value;
                updateMask();
                break;

            case "ui_date_date":
                options.ui_date_date = val.value.toString();
                updateTime();
                updateMask();
                break;
            case "ui_date_style":
                options.ui_date_style = val.value;
                updateMask();
                break;
            case "ui_date_year":
                options.ui_date_year = val.value.toString();
                updateMask();
                break;
            case "ui_date_order":
                options.ui_date_order = val.value.toString();
                updateMask();
                break;
            case "ui_date_monthname":
                options.ui_date_monthName = val.value;
                updateMask();
                break;
            case "ui_date_allcaps":
                options.ui_date_allCaps = val.value;
                updateMask();
                break;
            case "ui_date_delimiter":
                options.ui_date_delimiter = val.value.toString();
                updateMask();
                break;
            case "ui_date_scale":
                options.ui_date_scale = val.value;
                updateMask();
                break;
            case "ui_date_positionx":
                options.ui_date_positionX = val.value;
                updateMask();
                break;
            case "ui_date_positiony":
                options.ui_date_positionY = val.value;
                updateMask();
                break;

            case "ui_message_message":
                options.ui_message_message = val.value;
                updateMask();
                break;
            case "ui_message_text":
                options.ui_message_text = val.value;
                updateMask();
                break;
            case "ui_message_scale":
                options.ui_message_scale = val.value;
                updateMask();
                break;
            case "ui_message_positionx":
                options.ui_message_positionX = val.value;
                updateMask();
                break;
            case "ui_message_positiony":
                options.ui_message_positionY = val.value;
                updateMask();
                break;

            case "ui_other_codescommaseparated":
                options.codes = makeCodes(val.value);
                initialAnimation();
                break;
        }
    };

    window.addEventListener('resize', function () {
        updateCanvasSize();
        updateGrid();
        updateMask();
        updateFont();
        initialAnimation();
    }, false);

    //MARK: Variables
    let months = [
        ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"],
        ["Farvardin", "Ordibehesht", "Khordad", "Tir", "Mordad", "Shahrivar", "Mehr", "Aban", "Azar", "Dey", "Bahman", "Esfand"],
        ["Muharram", "Safar", "Rabi' al-Awwal", "Rabi' al-Thani", "Jumada al-Awwal", "Jumada al-Thani", "Rajab", "Sha'ban", "Ramadan", "Shawwal", "Dhu al-Qadah", "Dhu al-Hijjah"]
    ], dateDelimiters = ["", " ", "-", ".", "/"];
    let days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    let fonts = ["monospace", "consolas", "courier-bold", "neo-matrix"];
    let charsets = [
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ()._,-=+*/\\:;\'\"<>?!@#$%&^[]{}",
        "1234567890アァカサタナハマヤャラワガザダバパイィキシチニヒミリヰギジヂビピウゥクスツヌフムユュルグズブヅプエェケセテネヘメレヱゲゼデベペオォコソトノホモヨョロヲゴゾドボポヴッン日Z:・.\"=*+-<>¦｜_╌",
        "01",
        "0123456789ABCDEF",
        "|."
    ];
    var logo = null, logos = ["ipaf", "kali-1", "kali-2", "pardus", "ubuntu-1", "ubuntu-2", "windows-11", "windows-10-8", "windows-7", "visual-studio", "vs-code", "unity-1", "unity-2", "unreal", "python", "blazor", "docker", "flutter", "git", "blender", "angular", "c-sharp", "c-plus-plus", "qt"];
    var debug = document.getElementById("debug"), logs = [];
    var year = "", month = "", date = "", day = "", hour = "", minute = "";
    var startTime, now, then, elapsed, letters, columns, rows, drops, staticChars;
    var AudioTimeout = false, LastSoundTime = new Date(), isSilent = false, frequencyArray, frequencyArrayLength = 128, column_frequency;
    var column_hue, row_hue;
    var font_offset_y, font_offset_x;
    var maskDom = document.getElementById("mask");
    var mask = maskDom.getContext("2d");
    var colorOverlayDom = document.getElementById("color-overlay");
    var colorOverlay = colorOverlayDom.getContext("2d");
    var neoMatrixDom = document.getElementById("neo-matrix");
    var neoMatrix = neoMatrixDom.getContext("2d");

    updateCanvasSize();
    updateCharSet();
    updateTime();
    updateFont();
    startAnimating();

    function updateCanvasSize() {
        neoMatrixDom.height = window.innerHeight;
        neoMatrixDom.width = window.innerWidth;
        maskDom.height = window.innerHeight;
        maskDom.width = window.innerWidth;
        colorOverlayDom.height = window.innerHeight;
        colorOverlayDom.width = window.innerWidth;
    }

    //MARK: Logo
    function updateLogo() {
        logo = new Image();
        logo.onload = updateMask;

        switch (options.ui_logo_logo) {
            case "0": {
                logo = null;
                updateMask();
                break;
            }
            case "1": {
                logo.src = options.ui_logo_customLogo;
                break;
            }
            default: {
                logo.src = "images/" + logos[parseInt(options.ui_logo_logo) - 2] + ".svg";
            }
        }
    }

    //MARK: Time
    setInterval(() => {
        updateTime();
        if (options.ui_clock_clock != "0")
            updateMask();
    }, 60000);

    function updateTime() {
        var today = new Date();
        today.setHours(today.getHours() + options.ui_clock_dayLightSaving);

        switch (options.ui_date_date) {
            case "1":
                year = today.getFullYear();
                month = today.getMonth() + 1;
                date = today.getDate();
                break;
            case "2":
                var parts = today.toLocaleDateString('fa-IR-u-nu-latn').split("/");
                year = parseInt(parts[0]);
                month = parseInt(parts[1]);
                date = parseInt(parts[2]);
                break;
            case "3":
                var parts = today.toLocaleDateString('ar-SA-u-nu-latn').split("/");
                year = parseInt(parts[2]);
                month = parseInt(parts[1]);
                date = parseInt(parts[0]);
                break;
        }

        day = today.getDay();
        hour = today.getHours();
        minute = today.getMinutes();

        if (!options.ui_clock_24HourFormat && hour > 12) {
            hour = hour % 12;
            if (hour == 0)
                hour = 12;
        }
        if (hour < 10)
            hour = "0" + hour;
        if (minute < 10)
            minute = "0" + minute;

        hour = hour.toString();
        minute = minute.toString();
    }

    //MARK: Mask
    function updateMask() {
        clearStaticChars();

        mask.globalCompositeOperation = 'source-over';
        mask.clearRect(0, 0, neoMatrixDom.width, neoMatrixDom.height);
        mask.fillStyle = "rgba(0, 0, 0, " + options.trailLength + ")";
        mask.fillRect(0, 0, neoMatrixDom.width, neoMatrixDom.height);

        mask.globalCompositeOperation = 'destination-out';

        if (logo) {
            let logo_width = (neoMatrixDom.height / 2) * (logo.width / logo.height) * options.ui_logo_scale;
            let logo_height = (neoMatrixDom.height / 2) * options.ui_logo_scale;

            mask.drawImage(logo, neoMatrixDom.width / 2 - logo_width / 2 + options.ui_logo_positionX, neoMatrixDom.height / 2 - logo_height / 2 + options.ui_logo_positionY, logo_width, logo_height);

            colorOverlay.clearRect(0, 0, neoMatrixDom.width, neoMatrixDom.height);
            colorOverlay.drawImage(logo, neoMatrixDom.width / 2 - logo_width / 2 + options.ui_logo_positionX, neoMatrixDom.height / 2 - logo_height / 2 + options.ui_logo_positionY, logo_width, logo_height);
        }

        switch (options.ui_clock_clock) {
            case "1": {
                let clock = hour + ":" + minute;
                if (options.ui_clock_scale > 0) {
                    let center = [Math.floor((columns - 17 * options.ui_clock_scale) / 2), Math.floor((rows - 5 * options.ui_clock_scale) / 2)];
                    drawTextOnMask(clock, center[0] + options.ui_clock_positionX, center[1] + options.ui_clock_positionY, options.ui_clock_scale);
                } else {
                    let center = [Math.floor((columns - 5) / 2), Math.floor((rows - 1) / 2)];
                    drawTextOnMatrix(clock, center[0] + options.ui_clock_positionX, center[1] + options.ui_clock_positionY);
                }
                break;
            }
            case "2": {
                let clock = hour + "\\n" + minute;
                if (options.ui_clock_scale > 0) {
                    let center = [Math.floor((columns - 7 * options.ui_clock_scale) / 2), Math.floor((rows - 11 * options.ui_clock_scale) / 2)];
                    drawTextOnMask(clock, center[0] + options.ui_clock_positionX, center[1] + options.ui_clock_positionY, options.ui_clock_scale);
                } else {
                    let center = [Math.floor((columns - 2) / 2), Math.floor((rows - 2) / 2)];
                    drawTextOnMatrix(clock, center[0] + options.ui_clock_positionX, center[1] + options.ui_clock_positionY);
                }
                break;
            }
            case "3": {
                let h = hour.split("").join("\\n"), m = minute.split("").join("\\n");
                let clock = h + "\\n" + m;
                if (options.ui_clock_scale > 0) {
                    let center = [Math.floor((columns - 3 * options.ui_clock_scale) / 2), Math.floor((rows - 23 * options.ui_clock_scale) / 2)];
                    drawTextOnMask(clock, center[0] + options.ui_clock_positionX, center[1] + options.ui_clock_positionY, options.ui_clock_scale);
                } else {
                    let center = [Math.floor((columns - 1) / 2), Math.floor((rows - 4) / 2)];
                    drawTextOnMatrix(clock, center[0] + options.ui_clock_positionX, center[1] + options.ui_clock_positionY);
                }
                break;
            }
        }

        if (options.ui_day_day != "0") {
            var dayText = options.ui_day_allCaps ? days[day].toUpperCase() : days[day];
            if (options.ui_day_day == "2")
                dayText = dayText.substring(0, 3);
            if (options.ui_day_orientation)
                dayText = dayText.split("").join("\\n");
            if (options.ui_day_scale > 0) {
                let bb = getTextBoundingBox(dayText, options.ui_day_scale);
                let center = [Math.floor((columns - bb[0]) / 2), Math.floor((rows - bb[1]) / 2)];
                drawTextOnMask(dayText, center[0] + options.ui_day_positionX, center[1] + options.ui_day_positionY, options.ui_day_scale);
            } else {
                let cc = getCharsCount(dayText);
                let center = [Math.floor((columns - cc[0]) / 2), Math.floor((rows - cc[1]) / 2)];
                drawTextOnMatrix(dayText, center[0] + options.ui_day_positionX, center[1] + options.ui_day_positionY);
            }
        }

        if (options.ui_date_date != "0") {
            var text3 = date.toString(), text2, text1 = "", completeDate;
            if (text3.length < 2)
                text3 = "0" + text3;
            if (options.ui_date_monthName) {
                text2 = months[parseInt(options.ui_date_date) - 1][month - 1];
                if (options.ui_date_allCaps)
                    text2 = text2.toUpperCase();
            } else {
                text2 = month.toString();
                if (text2.length < 2)
                    text2 = "0" + text2;
            }
            switch (options.ui_date_year) {
                case "1": {
                    text1 = year.toString().substring(2, 4);
                    break;
                }
                case "2": {
                    text1 = year.toString();
                    break;
                }
            }

            if(options.ui_date_order == 1){
                let tmp = text2;
                text2 = text3;
                text3 = tmp;
            }

            let delimiter = dateDelimiters[parseInt(options.ui_date_delimiter)];

            switch (options.ui_date_style) {
                case "0":
                    completeDate = (text1.length > 0 ? [text1, text2, text3] : [text2, text3]).join(delimiter);
                    break;
                case "1":
                    completeDate = (text1.length > 0 ? [text1, text2, text3] : [text2, text3]).join("\\n");
                    break;
                case "2":
                    completeDate = (text1 + text2 + text3).split("").join("\\n");
                    break;
            }

            if (options.ui_date_scale > 0) {
                let bb = getTextBoundingBox(completeDate, options.ui_date_scale);
                let center = [Math.floor((columns - bb[0]) / 2), Math.floor((rows - bb[1]) / 2)];
                drawTextOnMask(completeDate, center[0] + options.ui_date_positionX, center[1] + options.ui_date_positionY, options.ui_date_scale);
            } else {
                let cc = getCharsCount(completeDate);
                let center = [Math.floor((columns - cc[0]) / 2), Math.floor((rows - cc[1]) / 2)];
                drawTextOnMatrix(completeDate, center[0] + options.ui_date_positionX, center[1] + options.ui_date_positionY);
            }
        }

        if (options.ui_message_message) {
            if (options.ui_message_scale > 0) {
                let bb = getTextBoundingBox(options.ui_message_text, options.ui_message_scale);
                let center = [Math.floor((columns - bb[0]) / 2), Math.floor((rows - bb[1]) / 2)];
                drawTextOnMask(options.ui_message_text, center[0] + options.ui_message_positionX, center[1] + options.ui_message_positionY, options.ui_message_scale);
            } else {
                let cc = getCharsCount(options.ui_message_text);
                let center = [Math.floor((columns - cc[0]) / 2), Math.floor((rows - cc[1]) / 2)];
                drawTextOnMatrix(options.ui_message_text, center[0] + options.ui_message_positionX, center[1] + options.ui_message_positionY);
            }
        }
    }

    function drawTextOnMatrix(text, x, y) {
        mask.fillStyle = "#FFF";
        lines = text.split("\\n");

        let cc = getCharsCount(text);

        x = clamp(0, columns - cc[0], x);
        y = clamp(0, rows - cc[1], y);

        for (let i = 0; i < lines.length; i++) {

            let sections = lines[i].split(" "), currentCharIndex = 0;
            for (let j = 0; j < sections.length; j++) {
                if (sections[j].length > 0)
                    mask.fillRect((x + currentCharIndex) * options.ui_font_size - font_offset_x, (y + i) * options.ui_font_size + font_offset_y, sections[j].length * options.ui_font_size, options.ui_font_size);
                currentCharIndex += sections[j].length + 1;
            }

            for (let j = 0; j < lines[i].length; j++)
                staticChars[x + j][y + i + 1] = lines[i][j] != " " ? lines[i][j] : null;
        }
    }

    function drawTextOnMask(text, x, y, scale) {
        mask.font = options.ui_font_size * 5 * scale + "px neo-matrix";
        mask.fillStyle = "#FFF";
        lines = text.split("\\n");

        for (let i = 0; i < lines.length; i++)
            mask.fillText(lines[i], options.ui_font_size * x - font_offset_x, options.ui_font_size * (y + ((6 * (i + 1)) - 1) * scale) + font_offset_y);
    }

    function getTextBoundingBox(text, scale) {
        let cc = getCharsCount(text);
        return [(cc[0] * 4 - 1) * scale, (cc[1] * 6 - 1) * scale];
    }

    function getCharsCount(text) {
        lines = text.split("\\n");
        var maxChars = 0;
        for (let i = 0; i < lines.length; i++)
            if (lines[i].length > maxChars)
                maxChars = lines[i].length;
        return [maxChars, lines.length];
    }

    function drawMask() {
        neoMatrix.globalCompositeOperation = 'source-over';
        neoMatrix.drawImage(maskDom, 0, 0);

        if (logo && options.ui_logo_preserveColor) {
            neoMatrix.globalCompositeOperation = 'source-atop';
            neoMatrix.drawImage(colorOverlayDom, 0, 0);
            neoMatrix.globalCompositeOperation = 'source-over';
        }
    }

    //MARK: Charset
    function updateCharSet() {
        if (options.ui_characters_charset == "0")
            letters = options.ui_characters_customCharset;
        else
            letters = charsets[parseInt(options.ui_characters_charset) - 1];

        letters = letters.split("");
    }

    //MARK: Font
    function updateFont() {
        var font_name;

        if (options.ui_font_font == "0")
            font_name = options.ui_font_customFont;
        else
            font_name = fonts[parseInt(options.ui_font_font) - 1];

        neoMatrix.font = options.ui_font_size + "px " + font_name;
        font_offset_y = options.ui_font_size / 8;
        font_offset_x = options.ui_font_size / 16;

        updateGrid();
        updateMask();
        initialAnimation();
    }

    //MARK: Grid
    function updateGrid() {
        columns = Math.floor(neoMatrixDom.width / options.ui_font_size);
        rows = Math.floor(neoMatrixDom.height / options.ui_font_size);
        column_hue = Math.floor(360 / columns);
        row_hue = Math.floor(360 / rows);
        column_frequency = frequencyArrayLength / (columns * 2);
        clearStaticChars();
    }

    function clearStaticChars() {
        staticChars = [];
        for (let i = 0; i < columns; i++) {
            staticChars[i] = [];
            for (let j = 0; j < rows; j++)
                staticChars[i][j] = null;
        }
    }

    //MARK: Initial Animation
    function initialAnimation() {
        drops = [];

        switch (options.ui_rain_initialAnimation) {
            case "0": {
                for (var i = 0; i < columns; i++) {
                    drops[i] = [];
                    for (var j = 0; j < options.ui_rain_dropCount; j++)
                        drops[i][j] = [rows + 1, 0, 0, "", 0];
                }
                break;
            }
            case "1": {
                for (var i = 0; i < columns; i++) {
                    drops[i] = [];
                    drops[i][0] = [1, 0, 0, "", 0];
                    for (var j = 1; j < options.ui_rain_dropCount; j++)
                        drops[i][j] = [rows + 1, 0, 0, "", 0];
                }
                break;
            }
            case "2": {
                for (var i = 0; i < columns; i++) {
                    drops[i] = [];
                    for (var j = 0; j < options.ui_rain_dropCount; j++)
                        drops[i][j] = [Math.floor(Math.random() * rows), 0, 0, "", 0];
                }
                break;
            }
        }
    }

    function startAnimating() {
        checkForUpdates();
        then = Date.now();
        startTime = then;
        loop();
    }

    function loop() {
        window.requestAnimationFrame(loop);
        now = Date.now();
        elapsed = now - then;
        if (elapsed > options.fpsInterval) {
            then = now - (elapsed % options.fpsInterval);
            drawMatrix();
        }
    }

    //MARK: Draw Matrix
    function drawMatrix() {
        drawMask();
        isSilent = true;

        for (var i = 0; i < drops.length; i++) {
            var probability = 0.975;
            var audio_lightness = 50;

            if (options.ui_audio_audioResponsive) {
                var frequency = Math.floor(i * column_frequency);
                var Volume = frequencyArray[frequency] + frequencyArray[frequency + (frequencyArrayLength / 2)];

                if (Volume > 0.01)
                    isSilent = false;

                if (!AudioTimeout || !options.ui_audio_silenceAnimation) {
                    probability = 1 - clamp(0, 1, (Volume * Volume * Volume * options.ui_audio_audioSensetivity));
                    audio_lightness = Math.floor(clamp(40, 80, Volume * 100 * options.ui_audio_audioSensetivity));
                }
            }

            var newDrop = true;
            for (var j = 0; j < options.ui_rain_dropCount; j++) {
                var character = calculateCharacter(drops[i][j], i);
                var lightness = audio_lightness;

                if (drops[i][j][1] > 0)
                    lightness = 100;

                if (options.ui_color_highlightFirstCharacter) {
                    neoMatrix.clearRect(i * options.ui_font_size - font_offset_x, ((drops[i][j][0] - 2) * options.ui_font_size) + font_offset_y, options.ui_font_size, options.ui_font_size);

                    var tmp = drops[i][j][0] - 1;
                    neoMatrix.fillStyle = calculateColor(i, tmp, drops[i][j][4]);
                    neoMatrix.fillText(drops[i][j][3], i * options.ui_font_size, tmp * options.ui_font_size);

                    neoMatrix.fillStyle = "#FFF";
                }
                else
                    neoMatrix.fillStyle = calculateColor(i, drops[i][j][0], lightness);

                neoMatrix.clearRect(i * options.ui_font_size, ((drops[i][j][0] - 1) * options.ui_font_size) + font_offset_y, options.ui_font_size, options.ui_font_size);
                drops[i][j][3] = character, drops[i][j][4] = lightness;
                neoMatrix.fillText(character, i * options.ui_font_size, drops[i][j][0] * options.ui_font_size);

                if (drops[i][j][0] > rows && Math.random() > probability && newDrop) {
                    drops[i][j] = [0, 0, 0, "", 0];
                    newDrop = false;
                }

                drops[i][j][0]++;
            }
        }

        if (options.ui_audio_silenceAnimation) {
            if (!isSilent) {
                AudioTimeout = false;
                LastSoundTime = new Date();
            } else if ((new Date() - LastSoundTime) > options.ui_audio_silenceTimeoutSeconds * 1000) {
                AudioTimeout = true;
            }
        }
    }

    //MARK: Calculate Character
    function calculateCharacter(dropItem, column) {
        if (staticChars[column][dropItem[0]])
            return staticChars[column][dropItem[0]];

        if (Math.random() > 0.995 && dropItem[1] == 0) {
            dropItem[1] = Math.floor(Math.random() * options.codes.length) + 1;
            dropItem[2] = dropItem[0];
        }

        if (dropItem[1] != 0) {
            var codeCharIndex = dropItem[0] - dropItem[2];
            if (codeCharIndex < options.codes[dropItem[1] - 1].length)
                return options.codes[dropItem[1] - 1][codeCharIndex];
            dropItem[1] = 0;
            dropItem[2] = 0;
        }

        return letters[Math.floor(Math.random() * letters.length)];
    }

    //MARK: Calculate Color
    function calculateColor(i, j, lightness) {
        var hue, offset = Math.floor(options.colorAnimationSpeed * then);

        switch (options.ui_color_colorMode) {
            //RGb cycle
            case "1": {
                hue = offset * row_hue;
                break;
            }
            //Vertical
            case "2": {
                hue = (j + offset) * row_hue;
                break;
            }
            //Horizontal
            case "3": {
                hue = (i + offset) * column_hue;
                break;
            }
            //Static
            default: {
                hue = options.matrixColor;
                break;
            }
        }

        return "hsl(" + hue + ", 100%, " + lightness + "%)";;
    }

    function calculateFpsInterval(fps) {
        return 1000 / fps;
    }

    function calculateTrailLength(value) {
        return map(value, 0.0, 1.0, 0.35, 0.02);
    }

    function calculateColorAnimationSpeed(value) {
        return map(value, -1, 1, 0.05, -0.05);
    }

    function makeCodes(codesText) {
        var codes = codesText.split(",");
        return codes;
    }

    //MARK: Helpers
    function Log(text) {
        debug.classList.remove("hide");
        void debug.offsetWidth;
        logs.push(text);
        if (logs.length > 10)
            logs.splice(0, 1);
        var tmp = "";
        logs.forEach(l => { tmp += l + "\n" });
        debug.innerText = tmp;
        debug.classList.add("hide");
    }

    function rgbToHue(color) {
        let tmp = color.map(function (c) {
            return Math.ceil(c * 255)
        });
        return rgbToHsl(...tmp)[0] * 360;
    }

    function hexToRgb(hex) {
        let result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return result ? {
            r: parseInt(result[2], 16),
            g: parseInt(result[3], 16),
            b: parseInt(result[4], 16)
        } : null;
    }

    function rgbToHsl(r, g, b) {
        r /= 255, g /= 255, b /= 255;

        var max = Math.max(r, g, b), min = Math.min(r, g, b);
        var h, s, l = (max + min) / 2;

        if (max == min) {
            h = s = 0; // achromatic
        } else {
            var d = max - min;
            s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

            switch (max) {
                case r: h = (g - b) / d + (g < b ? 6 : 0); break;
                case g: h = (b - r) / d + 2; break;
                case b: h = (r - g) / d + 4; break;
            }

            h /= 6;
        }

        return [h, s, l];
    }

    function map(value, from_a, from_b, to_a, to_b) {
        return (((value - from_a) * (to_b - to_a)) / (from_b - from_a)) + to_a;
    }

    function clamp(min, max, value) {
        if (value < min)
            return min;
        if (value > max)
            return max;
        return value;
    }

    function optionsToDict(options) {
        return options.reduce((acc, option) => {
            acc[option.label] = option.value;
            return acc;
        }, {});
    }

    function paramsToUrl(urlParams, paramDefaults, filter) {
        var defaults = new URLSearchParams(paramDefaults)
        var params = new URLSearchParams(urlParams)

        filter.forEach(key => {
            params.delete(key);
            defaults.delete(key);
        });

        defaults.forEach((value, key) => {
            if (params.get(key) === value)
                params.delete(key);
        });

        return window.location.origin + window.location.pathname + "?" + params.toString();
    }

    function copyToClipboard(text) {
        const el = document.createElement('textarea');
        el.value = text;
        el.setAttribute('readonly', '');
        document.body.appendChild(el);
        el.select();
        document.execCommand('copy');
        document.body.removeChild(el);
    }

    function getUrlParams() {
        urlParams = new URLSearchParams(window.location.search);
        if (urlParams.size == 0)
            return null;

        params = {};
        for (const [key, value] of urlParams)
            params[key] = value;

        return params;
    }
};

