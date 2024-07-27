using System.IO;
using System.Security.AccessControl;
using System.Security.Principal;
using SSSHU = Sucrose.Shared.Space.Helper.User;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Access
    {
        public static bool File(string filePath)
        {
            try
            {
                FileInfo fileInfo = new(filePath);

                if (!fileInfo.Exists)
                {
                    return false;
                }

                FileSecurity fileSecurity = fileInfo.GetAccessControl();

                AuthorizationRuleCollection rules = fileSecurity.GetAccessRules(true, true, typeof(NTAccount));

                WindowsPrincipal principal = SSSHU.GetPrincipal(SSSHU.GetIdentity());

                return Permissions(principal, rules, FileSystemRights.Read) && Permissions(principal, rules, FileSystemRights.Write) && Permissions(principal, rules, FileSystemRights.Delete);
            }
            catch
            {
                return false;
            }
        }

        public static bool Directory(string directoryPath)
        {
            try
            {
                DirectoryInfo directoryInfo = new(directoryPath);

                if (!directoryInfo.Exists)
                {
                    return false;
                }

                DirectorySecurity directorySecurity = directoryInfo.GetAccessControl();

                AuthorizationRuleCollection rules = directorySecurity.GetAccessRules(true, true, typeof(NTAccount));

                WindowsPrincipal principal = SSSHU.GetPrincipal(SSSHU.GetIdentity());

                return Permissions(principal, rules, FileSystemRights.Delete) && Permissions(principal, rules, FileSystemRights.ListDirectory) && Permissions(principal, rules, FileSystemRights.CreateDirectories);
            }
            catch
            {
                return false;
            }
        }

        private static bool Permissions(WindowsPrincipal principal, AuthorizationRuleCollection rules, FileSystemRights rights)
        {
            foreach (FileSystemAccessRule rule in rules)
            {
                if (principal.IsInRole(rule.IdentityReference.Value) && (rule.FileSystemRights & rights) != 0 && rule.AccessControlType == AccessControlType.Allow)
                {
                    return true;
                }
            }

            return false;
        }
    }
}