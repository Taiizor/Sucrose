using Newtonsoft.Json;
using SESMIEN = Sucrose.Shared.Engine.Manage.Internal.ExecuteNormal;
using SESMIET = Sucrose.Shared.Engine.Manage.Internal.ExecuteTask;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Properties
    {
        public static void ExecuteNormal(SESMIEN Function)
        {
            if (SSEMI.Initialized)
            {
                if (SSEMI.Properties.PropertyList.Any())
                {
                    foreach (KeyValuePair<string, object> Pair in SSEMI.Properties.PropertyList)
                    {
                        string Key = Pair.Key;
                        object Value = Pair.Value;

                        string Script = JsonConvert.SerializeObject(Value, Formatting.Indented);

                        Function(string.Format(SSEMI.Properties.PropertyListener, Key, Script));
                    }
                }
            }
        }

        public static void ExecuteTask(SESMIET Function)
        {
            if (SSEMI.Initialized)
            {
                SESMIEN AdaptedFunction = new(async (Script) =>
                {
                    await Function(Script);
                });

                ExecuteNormal(AdaptedFunction);
            }
        }
    }
}