using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using System.Net;
using SMCEC = Sucrose.Manager.Converter.EnumConverter;
using SMCIPAC = Sucrose.Manager.Converter.IPAddressConverter;
using SMHR = Sucrose.Manager.Helper.Reader;
using SMHV = Sucrose.Manager.Helper.Validator;
using SMHW = Sucrose.Manager.Helper.Writer;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMR = Sucrose.Memory.Readonly;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;

namespace Sucrose.Manager
{
    public class SettingManager2
    {
        private readonly string _settingsFilePath;
        private readonly JsonSerializerSettings _serializerSettings;

        public SettingManager2(string settingsFileName, Formatting formatting = Formatting.Indented, TypeNameHandling typeNameHandling = TypeNameHandling.None)
        {
            _settingsFilePath = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMR.SettingFolder, settingsFileName);

            Directory.CreateDirectory(Path.GetDirectoryName(_settingsFilePath));

            _serializerSettings = new JsonSerializerSettings
            {
                TypeNameHandling = typeNameHandling,
                Formatting = formatting,
                Converters =
                {
                    new SMCEC(),
                    new SMCIPAC(),
                    //new StringEnumConverter()
                }
            };

            ControlFile();
        }

        public T GetSetting<T>(string key, T back = default)
        {
            using Mutex Mutex = new(false, Path.GetFileName(_settingsFilePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch
                {
                    //
                }

                if (CheckFile())
                {
                    string json = SMHR.Read(_settingsFilePath).Result;

                    Settings settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);

                    if (settings != null && settings.Properties != null && settings.Properties.TryGetValue(key, out object value))
                    {
                        return ConvertToType<T>(value);
                    }
                }
            }
            finally
            {
                Mutex.ReleaseMutex();
            }

            return back;
        }

        public T GetSettingStable<T>(string key, T back = default)
        {
            using Mutex Mutex = new(false, Path.GetFileName(_settingsFilePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch
                {
                    //
                }

                if (CheckFile())
                {
                    string json = SMHR.Read(_settingsFilePath).Result;

                    Settings settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);

                    if (settings != null && settings.Properties != null && settings.Properties.TryGetValue(key, out object value))
                    {
                        return JsonConvert.DeserializeObject<T>(value.ToString());
                    }
                }
            }
            finally
            {
                Mutex.ReleaseMutex();
            }

            return back;
        }

        public T GetSettingAddress<T>(string key, T back = default)
        {
            using Mutex Mutex = new(false, Path.GetFileName(_settingsFilePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch
                {
                    //
                }

                if (CheckFile())
                {
                    string json = SMHR.Read(_settingsFilePath).Result;

                    Settings settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);

                    if (settings != null && settings.Properties != null && settings.Properties.TryGetValue(key, out object value))
                    {
                        return ConvertToType<T>(value);
                    }
                }
            }
            finally
            {
                Mutex.ReleaseMutex();
            }

            return back;
        }

        public void SetSetting<T>(string key, T value)
        {
            SetSetting(new KeyValuePair<string, T>[]
            {
                new(key, value)
            });
        }

        public void SetSetting<T>(KeyValuePair<string, T>[] pairs)
        {
            using Mutex Mutex = new(false, Path.GetFileName(_settingsFilePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch
                {
                    //
                }

                Settings settings;

                if (CheckFile())
                {
                    string json = SMHR.Read(_settingsFilePath).Result;
                    settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);
                }
                else
                {
                    settings = new Settings();
                }

                foreach (KeyValuePair<string, T> pair in pairs)
                {
                    settings.Properties[pair.Key] = ConvertToType<T>(pair.Value);
                }

                SMHW.Write(_settingsFilePath, JsonConvert.SerializeObject(settings, _serializerSettings));
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }

        public string ReadSetting()
        {
            using Mutex Mutex = new(false, Path.GetFileName(_settingsFilePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch
                {
                    //
                }

                return SMHR.Read(_settingsFilePath).Result;
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }

        public void ApplySetting()
        {
            using Mutex Mutex = new(false, Path.GetFileName(_settingsFilePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch
                {
                    //
                }

                Settings settings = new();

                SMHW.Write(_settingsFilePath, JsonConvert.SerializeObject(settings, _serializerSettings));
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }

        public bool CheckFile()
        {
            return File.Exists(_settingsFilePath);
        }

        public string SettingFile()
        {
            return _settingsFilePath;
        }

        private void ControlFile()
        {
            if (CheckFile())
            {
                string json = ReadSetting();

                if (string.IsNullOrEmpty(json))
                {
                    ApplySetting();
                }
                else if (!SMHV.Json(json))
                {
                    ApplySetting();
                }
                else
                {
                    try
                    {
                        Settings settings = JsonConvert.DeserializeObject<Settings>(json, _serializerSettings);

                        if (settings != null && settings.Properties != null)
                        {
                            return;
                        }
                        else
                        {
                            ApplySetting();
                        }
                    }
                    catch
                    {
                        ApplySetting();
                    }
                }
            }
            else
            {
                ApplySetting();
            }
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
            else if (type == typeof(string[]))
            {
                if (value is string[] array)
                {
                    return (T)(object)array;
                }
                else if (value is JArray jArray)
                {
                    return (T)(object)jArray.Select(jValue => (string)jValue).ToArray();
                }
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
            public Dictionary<string, object> Properties { get; set; } = new();
        }
    }
}