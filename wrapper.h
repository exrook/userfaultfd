#include <linux/userfaultfd.h>
#include <sys/ioctl.h>

const unsigned long int T_UFFD_API = UFFD_API;
#undef UFFD_API
const __u64 UFFD_API = T_UFFD_API;

const long int T_UFFDIO_API = UFFDIO_API;
#undef UFFDIO_API
const long int UFFDIO_API = T_UFFDIO_API;

const long int T_UFFDIO_REGISTER = UFFDIO_REGISTER;
#undef UFFDIO_REGISTER
const long int UFFDIO_REGISTER = T_UFFDIO_REGISTER;

const long int T_UFFDIO_UNREGISTER = UFFDIO_UNREGISTER;
#undef UFFDIO_UNREGISTER
const long int UFFDIO_UNREGISTER = T_UFFDIO_UNREGISTER;

const long int T_UFFDIO_WAKE = UFFDIO_WAKE;
#undef UFFDIO_WAKE
const long int UFFDIO_WAKE = T_UFFDIO_WAKE;

const long int T_UFFDIO_COPY = UFFDIO_COPY;
#undef UFFDIO_COPY
const long int UFFDIO_COPY = T_UFFDIO_COPY;

const long int T_UFFDIO_ZEROPAGE = UFFDIO_ZEROPAGE;
#undef UFFDIO_ZEROPAGE
const long int UFFDIO_ZEROPAGE = T_UFFDIO_ZEROPAGE;
