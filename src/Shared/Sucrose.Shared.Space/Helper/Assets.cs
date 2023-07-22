using System.IO;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Assets
    {
        public static string Get(string Assets)
        {
            return Path.Combine(SSSMI.This, Assets);
        }
    }
}