function mapAndApply(fromObject, toObject, mapper) {
    for (const [inputKey, inputValue] of Object.entries(fromObject)) {
        let mapEntry = mapper.properties[inputKey];
        if (!mapEntry)
            if (mapper.mapUndefined)
                mapEntry = {};
            else
                continue;

        const destinationKey = mapEntry.key || inputKey;
        const conversionFunc = mapEntry.convert || ((val) => val);
        const onChange = mapEntry.onChange || (() => { });

        const convertedValue = conversionFunc(inputValue);

        const valuesAreEqual = Array.isArray(toObject[destinationKey]) && Array.isArray(convertedValue)
            ? arraysAreEqual(toObject[destinationKey], convertedValue)
            : toObject[destinationKey] === convertedValue;

        if (!valuesAreEqual) {
            toObject[destinationKey] = convertedValue;
            onChange();
        }
    }
}