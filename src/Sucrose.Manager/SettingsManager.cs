using Newtonsoft.Json;
using Sucrose.Memory;
using System.Collections.Generic;
using System.IO;
using System.Net;

namespace Sucrose.Manager
{
    public class SettingsManager
    {
        private readonly string _settingsFilePath;
        private readonly JsonSerializerSettings _serializerSettings;

        public SettingsManager(string settingsFileName, Formatting formatting = Formatting.Indented, TypeNameHandling typeNameHandling = TypeNameHandling.None)
        {
            _settingsFilePath = Path.Combine(Readonly.AppDataPath, Readonly.AppName, settingsFileName);

            Directory.CreateDirectory(Path.GetDirectoryName(_settingsFilePath));

            _serializerSettings = new JsonSerializerSettings
            {
                TypeNameHandling = typeNameHandling,
                Formatting = formatting,
                Converters = new[]
                {
                    new IPAddressConverter()
                }
            };
        }

        public T GetSetting<T>(string key)
        {
            if (File.Exists(_settingsFilePath))
            {
                string json = File.ReadAllText(_settingsFilePath);
                
                Settings? settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);
                
                if (settings.Properties.TryGetValue(key, out object value))
                {
                    return (T)value;
                }
            }

            return default;
        }

        public T GetSettingStable<T>(string key)
        {
            if (File.Exists(_settingsFilePath))
            {
                string json = File.ReadAllText(_settingsFilePath);
                
                Settings settings = JsonConvert.DeserializeObject<Settings>(json);
                
                if (settings.Properties.TryGetValue(key, out object value))
                {
                    return JsonConvert.DeserializeObject<T>(value.ToString());
                }
            }

            return default;
        }

        public T GetSettingAddress<T>(string key)
        {
            if (File.Exists(_settingsFilePath))
            {
                string json = File.ReadAllText(_settingsFilePath);
                
                Settings settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);
                
                if (settings.Properties.TryGetValue(key, out object value))
                {
                    return ConvertToType<T>(value);
                }
            }

            return default;
        }

        public void SetSetting<T>(string key, T value)
        {
            Settings settings;

            if (File.Exists(_settingsFilePath))
            {
                string json = File.ReadAllText(_settingsFilePath);
                settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);
            }
            else
            {
                settings = new Settings();
            }

            settings.Properties[key] = ConvertToType<T>(value);

            string serializedSettings = JsonConvert.SerializeObject(settings, _serializerSettings);

            File.WriteAllText(_settingsFilePath, serializedSettings);
        }

        private T ConvertToType<T>(object value)
        {
            if (typeof(T) == typeof(IPAddress))
            {
                return (T)(object)IPAddress.Parse(value.ToString());
            }

            return (T)value;
        }

        private class Settings
        {
            public Dictionary<string, object> Properties { get; set; } = new Dictionary<string, object>();
        }
    }
}