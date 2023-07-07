namespace Sucrose.CommandLine.Helper
{
    internal static class Parse
    {
        public static T ArgumentValue<T>(string ArgValue)
        {
            if (typeof(T) == typeof(int))
            {
                if (int.TryParse(ArgValue, out int IntValue))
                {
                    return (T)(object)IntValue;
                }
            }
            else if (typeof(T) == typeof(bool))
            {
                if (bool.TryParse(ArgValue, out bool BoolValue))
                {
                    return (T)(object)BoolValue;
                }
            }

            return (T)(object)ArgValue;
        }

        public static T ArgumentValueStable<T>(string ArgValue)
        {
            Type TargetType = typeof(T);

            if (TargetType == typeof(int))
            {
                if (int.TryParse(ArgValue, out int IntValue))
                {
                    return (T)(object)IntValue;
                }
            }
            else if (TargetType == typeof(bool))
            {
                if (bool.TryParse(ArgValue, out bool BoolValue))
                {
                    return (T)(object)BoolValue;
                }
            }
            else if (TargetType == typeof(string))
            {
                return (T)(object)ArgValue;
            }

            return default;
        }
    }
}
