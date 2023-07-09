namespace Sucrose.Commandog.Helper
{
    internal static class Miscellaneous
    {
        public static Type GetType(string Variable)
        {
            if (bool.TryParse(Variable, out bool _))
            {
                return typeof(bool);
            }

            if (int.TryParse(Variable, out int _))
            {
                return typeof(int);
            }

            return typeof(string);
        }
    }
}