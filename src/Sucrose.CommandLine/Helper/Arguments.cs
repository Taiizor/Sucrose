using System.Diagnostics;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SWHWS = Skylark.Wing.Helper.WindowsStartup;

namespace Sucrose.CommandLine.Helper
{
    internal class Arguments
    {
        public static void Parse(string[] Arguments)
        {
            if (Arguments.Any())
            {
                for (int Count = 0; Count < Arguments.Length; Count++)
                {
                    string Argument = Arguments[Count];

                    if (Argument.StartsWith(SMR.StartCommand) && Argument.Contains(SMR.ValueSeparatorChar))
                    {

#if NET6_0_OR_GREATER
                        string[] ArgumentParts = Argument[1..].Split(SMR.ValueSeparatorChar);
#else
                        string[] ArgumentParts = Argument.Substring(1).Split(SMR.ValueSeparatorChar);
#endif

                        if (ArgumentParts.Length >= 2)
                        {
                            string Name = ArgumentParts[0];

#if NET6_0_OR_GREATER
                            List<string> Values = new(ArgumentParts[1..]);
#else
                            List<string> Values = new();

                            for (int Index = 1; Index < ArgumentParts.Length; Index++)
                            {
                                Values.Add(ArgumentParts[Index]);
                            }
#endif

                            if (Enum.TryParse(Name, true, out SSECT Command))
                            {
                                switch (Command)
                                {
                                    case SSECT.Test:
                                        Console.WriteLine("Test değerleri:");

                                        foreach (string value in Values)
                                        {
                                            Type Types = GetType(value);

                                            if (Types == typeof(int))
                                            {
                                                Console.WriteLine(ParseArgumentValue<int>(value));
                                            }
                                            else if (Types == typeof(bool))
                                            {
                                                Console.WriteLine(ParseArgumentValue<bool>(value));
                                            }
                                            else if (Types == typeof(string))
                                            {
                                                Console.WriteLine(ParseArgumentValue<string>(value));
                                            }
                                        }
                                        break;
                                    case SSECT.Report:
                                        Process.Start(ParseArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Startup:
                                        SWHWS.SetStartup(ParseArgumentValue<string>(Values[0]), ParseArgumentValue<string>(Values[1]), ParseArgumentValue<bool>(Values[2]));
                                        break;
                                    case SSECT.Interface:
                                        Process.Start(ParseArgumentValue<string>(Values[0]));
                                        break;
                                    default:
                                        break;
                                }
                            }
                        }
                    }
                }
            }
        }

        private static T ParseArgumentValue<T>(string argValue)
        {
            if (typeof(T) == typeof(int))
            {
                if (int.TryParse(argValue, out int intValue))
                {
                    return (T)(object)intValue;
                }
            }
            else if (typeof(T) == typeof(bool))
            {
                if (bool.TryParse(argValue, out bool boolValue))
                {
                    return (T)(object)boolValue;
                }
            }

            return (T)(object)argValue;
        }

        private static T ParseArgumentValue2<T>(string argValue)
        {
            Type TargetType = typeof(T);

            if (TargetType == typeof(int))
            {
                if (int.TryParse(argValue, out int intValue))
                {
                    return (T)(object)intValue;
                }
            }
            else if (TargetType == typeof(bool))
            {
                if (bool.TryParse(argValue, out bool boolValue))
                {
                    return (T)(object)boolValue;
                }
            }
            else if (TargetType == typeof(string))
            {
                return (T)(object)argValue;
            }

            return default;
        }

        private static Type GetType(string variable)
        {
            if (bool.TryParse(variable, out bool _))
            {
                return typeof(bool);
            }

            if (int.TryParse(variable, out int _))
            {
                return typeof(int);
            }

            return typeof(string);
        }
    }
}