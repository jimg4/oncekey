function delayValid(value) {
    return /^(0|[1-9][0-9]{0,2})$/.test(String(value)) && Number(value) <= 200;
}
function buildConfig(globalText, overrides) {
    if (!delayValid(globalText)) throw new Error("Global delay must be a whole number from 0 to 200 ms.");
    var result = {delay_ms: Number(globalText), key_delays_ms: {}};
    Object.keys(overrides).forEach(function(key) {
        if (!/^[1-9][0-9]*$/.test(key) || Number(key) > 767)
            throw new Error("Invalid key code: " + key);
        var value = String(overrides[key]);
        if (value === "") return;
        if (!delayValid(value)) throw new Error("Key " + key + ": enter 0–200 ms or leave blank to use the global delay.");
        result.key_delays_ms[key] = Number(value);
    });
    return result;
}
function keyLabel(name) {
    var names = {LEFTMETA:"Left Command", RIGHTMETA:"Right Command", LEFTALT:"Left Option", RIGHTALT:"Right Option", LEFTCTRL:"Left Control", RIGHTCTRL:"Right Control", LEFTSHIFT:"Left Shift", RIGHTSHIFT:"Right Shift", CAPSLOCK:"Caps Lock", BACKSPACE:"Delete / Backspace", DELETE:"Forward Delete", SPACE:"Space", ESC:"Escape", ENTER:"Return", LEFT:"Left arrow", RIGHT:"Right arrow", UP:"Up arrow", DOWN:"Down arrow"};
    return names[name] || name.replace(/_/g, " ");
}
