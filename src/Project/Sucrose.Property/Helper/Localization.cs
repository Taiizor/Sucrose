using SMMM = Sucrose.Manager.Manage.Manager;
using SPMI = Sucrose.Property.Manage.Internal;

namespace Sucrose.Property.Helper
{
    internal static class Localization
    {
        public static string Convert(string Key)
        {
            if (SPMI.Properties.PropertyLocalization != null && SPMI.Properties.PropertyLocalization.Any())
            {
                if (SPMI.Properties.PropertyLocalization.TryGetValue(SMMM.Culture, out Dictionary<string, string> Pairs) || SPMI.Properties.PropertyLocalization.TryGetValue(SMMM.Culture.ToLower(), out Pairs) || SPMI.Properties.PropertyLocalization.TryGetValue(SMMM.Culture.ToLowerInvariant(), out Pairs))
                {
                    if (Pairs != null && Pairs.TryGetValue(Key, out string Value))
                    {
                        return Value;
                    }
                }

                if (SPMI.Properties.PropertyLocalization.TryGetValue(SPMI.Properties.PropertyLocalization.First().Key, out Pairs))
                {
                    if (Pairs != null && Pairs.TryGetValue(Key, out string Value))
                    {
                        return Value;
                    }
                }
            }

            return Key;
        }

        public static string[] Convert(string[] Key)
        {
            for (int Count = 0; Count < Key.Length; Count++)
            {
                Key[Count] = Convert(Key[Count]);
            }

            return Key;
        }
    }
}