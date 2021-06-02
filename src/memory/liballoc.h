#ifndef _LIBALLOC_H
#define _LIBALLOC_H

/** \defgroup ALLOCHOOKS liballoc hooks
 *
 * These are the OS specific functions which need to
 * be implemented on any platform that the library
 * is expected to work on.
 */

/** @{ */

#include <stddef.h>
#include <stdint.h>

// If we are told to not define our own size_t, then we skip the define.
//#define _HAVE_UINTPTR_T
//typedef	unsigned long	uintptr_t;

//This lets you prefix malloc and friends
#define PREFIX(func)		lalloc_ ## func

/** A structure found at the top of all system allocated
 * memory blocks. It details the usage of the memory block.
 */
struct liballoc_major
{
    struct liballoc_major *prev;		///< Linked list information.
    struct liballoc_major *next;		///< Linked list information.
    unsigned int pages;					///< The number of pages in the block.
    unsigned long long size;					///< The number of pages in the block.
    unsigned long long  usage;					///< The number of bytes used in the block.
    struct liballoc_minor *first;		///< A pointer to the first allocated memory in the block.
};


/** This is a structure found at the beginning of all
 * sections in a major block which were allocated by a
 * malloc, calloc, realloc call.
 */
struct	liballoc_minor
{
    struct liballoc_minor *prev;		///< Linked list information.
    struct liballoc_minor *next;		///< Linked list information.
    struct liballoc_major *block;		///< The owning block. A pointer to the major structure.
    unsigned int magic;					///< A magic number to idenfity correctness.
    unsigned long long size; 					///< The size of the memory allocated. Could be 1 byte or more.
    unsigned long long req_size;				///< The size of memory requested.
};

typedef struct process_heap_inner
{
    struct liballoc_major *root;
    struct liballoc_major *best_bet;
    unsigned long long allocated;
    unsigned long long inuse;
    unsigned long long warning_count;
    unsigned long long error_count;
    unsigned long long possible_overruns;
} process_heap_inner;

/** This is the hook into the local system which allocates pages. It
 * accepts an integer parameter which is the number of pages
 * required.  The page size was set up in the liballoc_init function.
 *
 * \return NULL if the pages were not allocated.
 * \return A pointer to the allocated memory.
 */
extern void* liballoc_alloc(size_t);

/** This frees previously allocated memory. The void* parameter passed
 * to the function is the exact same value returned from a previous
 * liballoc_alloc call.
 *
 * The integer value is the number of pages to free.
 *
 * \return 0 if the memory was successfully freed.
 */
extern int liballoc_free(void*,size_t);

extern void    *PREFIX(malloc)(process_heap_inner *, size_t);				///< The standard function.
extern void    *PREFIX(realloc)(process_heap_inner *, void *, size_t);		///< The standard function.
extern void    *PREFIX(calloc)(process_heap_inner *, size_t, size_t);		///< The standard function.
extern void     PREFIX(free)(process_heap_inner *, void *);					///< The standard function.

/** @} */

#endif