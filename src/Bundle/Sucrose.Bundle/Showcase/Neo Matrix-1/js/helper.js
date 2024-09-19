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

function preProcessProperties(properties) {
    const transformedProperties = {};
    for (const [key, value] of Object.entries(properties))
        transformedProperties[key] = value.value;
    return transformedProperties;
}

function preProcessPreset(preset) {
    const transformedProperties = {};
    for (const [key, value] of Object.entries(preset))
        transformedProperties[key.toLowerCase()] = value;
    return transformedProperties;
}

function arraysAreEqual(arr1, arr2) {
    if (!Array.isArray(arr1) || !Array.isArray(arr2))
        return false;
    if (arr1.length !== arr2.length)
        return false;
    for (let i = 0; i < arr1.length; i++)
        if (arr1[i] !== arr2[i])
            return false;
    return true;
}