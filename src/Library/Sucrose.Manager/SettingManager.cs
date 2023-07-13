using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using System.Net;
using SMCIPAC = Sucrose.Manager.Converter.IPAddressConverter;
using SMHR = Sucrose.Manager.Helper.Reader;
using SMHW = Sucrose.Manager.Helper.Writer;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Manager
{
    public class SettingManager
    {
        private static object lockObject = new();
        private readonly string _settingsFilePath;
        private readonly ReaderWriterLockSlim _lock;
        private readonly JsonSerializerSettings _serializerSettings;

        public SettingManager(string settingsFileName, Formatting formatting = Formatting.Indented, TypeNameHandling typeNameHandling = TypeNameHandling.None)
        {
            _settingsFilePath = Path.Combine(SMR.AppDataPath, SMR.AppName, settingsFileName);

            Directory.CreateDirectory(Path.GetDirectoryName(_settingsFilePath));

            _serializerSettings = new JsonSerializerSettings
            {
                TypeNameHandling = typeNameHandling,
                Formatting = formatting,
                Converters =
                {
                    //new SMCEC(),
                    new SMCIPAC(),
                    //new StringEnumConverter(),
                }
            };

            _lock = new ReaderWriterLockSlim(); //ReaderWriterLock
        }

        public T GetSetting<T>(string key, T back = default)
        {
            _lock.EnterReadLock();

            try
            {
                lock (lockObject)
                {
                    if (File.Exists(_settingsFilePath))
                    {
                        string json = SMHR.Read(_settingsFilePath);

                        Settings settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);

                        if (settings.Properties.TryGetValue(key, out object value))
                        {
                            return ConvertToType<T>(value);
                        }
                    }
                }
            }
            finally
            {
                _lock.ExitReadLock();
            }

            return back;
        }

        public T GetSettingStable<T>(string key, T back = default)
        {
            _lock.EnterReadLock();

            try
            {
                lock (lockObject)
                {
                    if (File.Exists(_settingsFilePath))
                    {
                        string json = SMHR.Read(_settingsFilePath);

                        Settings settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);

                        if (settings.Properties.TryGetValue(key, out object value))
                        {
                            return JsonConvert.DeserializeObject<T>(value.ToString());
                        }
                    }
                }
            }
            finally
            {
                _lock.ExitReadLock();
            }

            return back;
        }

        public T GetSettingAddress<T>(string key, T back = default)
        {
            _lock.EnterReadLock();

            try
            {
                lock (lockObject)
                {
                    if (File.Exists(_settingsFilePath))
                    {
                        string json = SMHR.Read(_settingsFilePath);

                        Settings settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);

                        if (settings.Properties.TryGetValue(key, out object value))
                        {
                            return ConvertToType<T>(value);
                        }
                    }
                }
            }
            finally
            {
                _lock.ExitReadLock();
            }

            return back;
        }

        public void SetSetting<T>(string key, T value)
        {
            _lock.EnterWriteLock();

            try
            {
                lock (lockObject)
                {
                    Settings settings;

                    if (File.Exists(_settingsFilePath))
                    {
                        string json = SMHR.Read(_settingsFilePath);
                        settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);
                    }
                    else
                    {
                        settings = new Settings();
                    }

                    settings.Properties[key] = ConvertToType<T>(value);

                    SMHW.Write(_settingsFilePath, JsonConvert.SerializeObject(settings, _serializerSettings));
                }
            }
            finally
            {
                _lock.ExitWriteLock();
            }
        }

        public bool CheckFile()
        {
            return File.Exists(_settingsFilePath);
        }

        private T ConvertToType<T>(object value)
        {
            Type type = typeof(T);

            if (type == typeof(IPAddress))
            {
                return (T)(object)IPAddress.Parse(value.ToString());
            }
            else if (type == typeof(Uri))
            {
                return (T)(object)new Uri(value.ToString());
            }
            else if (type.IsEnum)
            {
                return (T)Enum.Parse(type, value.ToString());
            }
            else if (type == typeof(KeyValuePair<string, string>))
            {
                string[] parts = value.ToString().Split(':');

                return (T)(object)new KeyValuePair<string, string>(parts[0].Trim(), parts[1].Trim());
            }
            else if (type == typeof(List<string>))
            {
                if (value is List<string> list)
                {
                    return (T)(object)list;
                }
                else if (value is JArray jArray)
                {
                    return (T)(object)jArray.Select(jValue => (string)jValue).ToList();
                }
            }
            else if (type == typeof(Dictionary<string, string>))
            {
                try
                {
                    return JsonConvert.DeserializeObject<T>(value.ToString());
                }
                catch
                {
                    return JsonConvert.DeserializeObject<T>(JsonConvert.SerializeObject(value));
                }
            }

            return (T)value;
        }

        private class Settings
        {
            public Dictionary<string, object> Properties { get; set; } = new Dictionary<string, object>();
        }
    }
}